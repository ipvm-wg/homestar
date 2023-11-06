(component
  (core module (;0;)
    (type (;0;) (func))
    (type (;1;) (func (param i32) (result i32)))
    (type (;2;) (func (param i32 i32) (result i32)))
    (type (;3;) (func (param i32 i32)))
    (type (;4;) (func (param i32)))
    (type (;5;) (func (param i32 i32 i32 i32) (result i32)))
    (type (;6;) (func (param i32 i32 i32) (result i32)))
    (func (;0;) (type 0))
    (func (;1;) (type 1) (param i32) (result i32)
      block ;; label = @1
        i32.const 0
        i32.load8_u offset=1049029
        br_if 0 (;@1;)
        call 0
        i32.const 0
        i32.const 1
        i32.store8 offset=1049029
      end
      local.get 0
      i32.const 2
      i32.add
    )
    (func (;2;) (type 2) (param i32 i32) (result i32)
      (local i32 i32 i32 i32 i32)
      i32.const 0
      local.set 2
      block ;; label = @1
        i32.const -65587
        local.get 0
        i32.const 16
        local.get 0
        i32.const 16
        i32.gt_u
        select
        local.tee 0
        i32.sub
        local.get 1
        i32.le_u
        br_if 0 (;@1;)
        local.get 0
        i32.const 16
        local.get 1
        i32.const 11
        i32.add
        i32.const -8
        i32.and
        local.get 1
        i32.const 11
        i32.lt_u
        select
        local.tee 3
        i32.add
        i32.const 12
        i32.add
        call 3
        local.tee 1
        i32.eqz
        br_if 0 (;@1;)
        local.get 1
        i32.const -8
        i32.add
        local.set 2
        block ;; label = @2
          block ;; label = @3
            local.get 0
            i32.const -1
            i32.add
            local.tee 4
            local.get 1
            i32.and
            br_if 0 (;@3;)
            local.get 2
            local.set 0
            br 1 (;@2;)
          end
          local.get 1
          i32.const -4
          i32.add
          local.tee 5
          i32.load
          local.tee 6
          i32.const -8
          i32.and
          local.get 4
          local.get 1
          i32.add
          i32.const 0
          local.get 0
          i32.sub
          i32.and
          i32.const -8
          i32.add
          local.tee 1
          i32.const 0
          local.get 0
          local.get 1
          local.get 2
          i32.sub
          i32.const 16
          i32.gt_u
          select
          i32.add
          local.tee 0
          local.get 2
          i32.sub
          local.tee 1
          i32.sub
          local.set 4
          block ;; label = @3
            local.get 6
            i32.const 3
            i32.and
            i32.eqz
            br_if 0 (;@3;)
            local.get 0
            local.get 0
            i32.load offset=4
            i32.const 1
            i32.and
            local.get 4
            i32.or
            i32.const 2
            i32.or
            i32.store offset=4
            local.get 0
            local.get 4
            i32.add
            local.tee 4
            local.get 4
            i32.load offset=4
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 5
            local.get 5
            i32.load
            i32.const 1
            i32.and
            local.get 1
            i32.or
            i32.const 2
            i32.or
            i32.store
            local.get 2
            local.get 1
            i32.add
            local.tee 4
            local.get 4
            i32.load offset=4
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 2
            local.get 1
            call 4
            br 1 (;@2;)
          end
          local.get 2
          i32.load
          local.set 2
          local.get 0
          local.get 4
          i32.store offset=4
          local.get 0
          local.get 2
          local.get 1
          i32.add
          i32.store
        end
        block ;; label = @2
          local.get 0
          i32.load offset=4
          local.tee 1
          i32.const 3
          i32.and
          i32.eqz
          br_if 0 (;@2;)
          local.get 1
          i32.const -8
          i32.and
          local.tee 2
          local.get 3
          i32.const 16
          i32.add
          i32.le_u
          br_if 0 (;@2;)
          local.get 0
          local.get 1
          i32.const 1
          i32.and
          local.get 3
          i32.or
          i32.const 2
          i32.or
          i32.store offset=4
          local.get 0
          local.get 3
          i32.add
          local.tee 1
          local.get 2
          local.get 3
          i32.sub
          local.tee 3
          i32.const 3
          i32.or
          i32.store offset=4
          local.get 0
          local.get 2
          i32.add
          local.tee 2
          local.get 2
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 1
          local.get 3
          call 4
        end
        local.get 0
        i32.const 8
        i32.add
        local.set 2
      end
      local.get 2
    )
    (func (;3;) (type 1) (param i32) (result i32)
      (local i32 i32 i32 i32 i32 i32 i32 i32 i64)
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  local.get 0
                  i32.const 245
                  i32.lt_u
                  br_if 0 (;@6;)
                  i32.const 0
                  local.set 1
                  local.get 0
                  i32.const -65587
                  i32.ge_u
                  br_if 5 (;@1;)
                  local.get 0
                  i32.const 11
                  i32.add
                  local.tee 0
                  i32.const -8
                  i32.and
                  local.set 2
                  i32.const 0
                  i32.load offset=1048988
                  local.tee 3
                  i32.eqz
                  br_if 4 (;@2;)
                  i32.const 0
                  local.set 4
                  block ;; label = @7
                    local.get 2
                    i32.const 256
                    i32.lt_u
                    br_if 0 (;@7;)
                    i32.const 31
                    local.set 4
                    local.get 2
                    i32.const 16777215
                    i32.gt_u
                    br_if 0 (;@7;)
                    local.get 2
                    i32.const 6
                    local.get 0
                    i32.const 8
                    i32.shr_u
                    i32.clz
                    local.tee 0
                    i32.sub
                    i32.shr_u
                    i32.const 1
                    i32.and
                    local.get 0
                    i32.const 1
                    i32.shl
                    i32.sub
                    i32.const 62
                    i32.add
                    local.set 4
                  end
                  i32.const 0
                  local.get 2
                  i32.sub
                  local.set 1
                  block ;; label = @7
                    local.get 4
                    i32.const 2
                    i32.shl
                    i32.const 1048576
                    i32.add
                    i32.load
                    local.tee 5
                    br_if 0 (;@7;)
                    i32.const 0
                    local.set 0
                    i32.const 0
                    local.set 6
                    br 2 (;@5;)
                  end
                  i32.const 0
                  local.set 0
                  local.get 2
                  i32.const 0
                  i32.const 25
                  local.get 4
                  i32.const 1
                  i32.shr_u
                  i32.sub
                  i32.const 31
                  i32.and
                  local.get 4
                  i32.const 31
                  i32.eq
                  select
                  i32.shl
                  local.set 7
                  i32.const 0
                  local.set 6
                  loop ;; label = @7
                    block ;; label = @8
                      local.get 5
                      i32.load offset=4
                      i32.const -8
                      i32.and
                      local.tee 8
                      local.get 2
                      i32.lt_u
                      br_if 0 (;@8;)
                      local.get 8
                      local.get 2
                      i32.sub
                      local.tee 8
                      local.get 1
                      i32.ge_u
                      br_if 0 (;@8;)
                      local.get 8
                      local.set 1
                      local.get 5
                      local.set 6
                      local.get 8
                      br_if 0 (;@8;)
                      i32.const 0
                      local.set 1
                      local.get 5
                      local.set 6
                      local.get 5
                      local.set 0
                      br 4 (;@4;)
                    end
                    local.get 5
                    i32.const 20
                    i32.add
                    i32.load
                    local.tee 8
                    local.get 0
                    local.get 8
                    local.get 5
                    local.get 7
                    i32.const 29
                    i32.shr_u
                    i32.const 4
                    i32.and
                    i32.add
                    i32.const 16
                    i32.add
                    i32.load
                    local.tee 5
                    i32.ne
                    select
                    local.get 0
                    local.get 8
                    select
                    local.set 0
                    local.get 7
                    i32.const 1
                    i32.shl
                    local.set 7
                    local.get 5
                    i32.eqz
                    br_if 2 (;@5;)
                    br 0 (;@7;)
                  end
                end
                block ;; label = @6
                  i32.const 0
                  i32.load offset=1048984
                  local.tee 7
                  i32.const 16
                  local.get 0
                  i32.const 11
                  i32.add
                  i32.const -8
                  i32.and
                  local.get 0
                  i32.const 11
                  i32.lt_u
                  select
                  local.tee 2
                  i32.const 3
                  i32.shr_u
                  local.tee 1
                  i32.shr_u
                  local.tee 0
                  i32.const 3
                  i32.and
                  i32.eqz
                  br_if 0 (;@6;)
                  block ;; label = @7
                    block ;; label = @8
                      local.get 0
                      i32.const -1
                      i32.xor
                      i32.const 1
                      i32.and
                      local.get 1
                      i32.add
                      local.tee 2
                      i32.const 3
                      i32.shl
                      local.tee 5
                      i32.const 1048728
                      i32.add
                      i32.load
                      local.tee 0
                      i32.const 8
                      i32.add
                      local.tee 6
                      i32.load
                      local.tee 1
                      local.get 5
                      i32.const 1048720
                      i32.add
                      local.tee 5
                      i32.eq
                      br_if 0 (;@8;)
                      local.get 1
                      local.get 5
                      i32.store offset=12
                      local.get 5
                      local.get 1
                      i32.store offset=8
                      br 1 (;@7;)
                    end
                    i32.const 0
                    local.get 7
                    i32.const -2
                    local.get 2
                    i32.rotl
                    i32.and
                    i32.store offset=1048984
                  end
                  local.get 0
                  local.get 2
                  i32.const 3
                  i32.shl
                  local.tee 2
                  i32.const 3
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 2
                  i32.add
                  local.tee 0
                  local.get 0
                  i32.load offset=4
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 6
                  return
                end
                local.get 2
                i32.const 0
                i32.load offset=1048992
                i32.le_u
                br_if 3 (;@2;)
                block ;; label = @6
                  block ;; label = @7
                    block ;; label = @8
                      block ;; label = @9
                        block ;; label = @10
                          block ;; label = @11
                            block ;; label = @12
                              local.get 0
                              br_if 0 (;@12;)
                              i32.const 0
                              i32.load offset=1048988
                              local.tee 0
                              i32.eqz
                              br_if 10 (;@2;)
                              local.get 0
                              i32.const 0
                              local.get 0
                              i32.sub
                              i32.and
                              i32.ctz
                              i32.const 2
                              i32.shl
                              i32.const 1048576
                              i32.add
                              i32.load
                              local.tee 6
                              i32.load offset=4
                              i32.const -8
                              i32.and
                              local.set 1
                              block ;; label = @13
                                local.get 6
                                i32.load offset=16
                                local.tee 0
                                br_if 0 (;@13;)
                                local.get 6
                                i32.const 20
                                i32.add
                                i32.load
                                local.set 0
                              end
                              local.get 1
                              local.get 2
                              i32.sub
                              local.set 5
                              block ;; label = @13
                                local.get 0
                                i32.eqz
                                br_if 0 (;@13;)
                                loop ;; label = @14
                                  local.get 0
                                  i32.load offset=4
                                  i32.const -8
                                  i32.and
                                  local.get 2
                                  i32.sub
                                  local.tee 8
                                  local.get 5
                                  i32.lt_u
                                  local.set 7
                                  block ;; label = @15
                                    local.get 0
                                    i32.load offset=16
                                    local.tee 1
                                    br_if 0 (;@15;)
                                    local.get 0
                                    i32.const 20
                                    i32.add
                                    i32.load
                                    local.set 1
                                  end
                                  local.get 8
                                  local.get 5
                                  local.get 7
                                  select
                                  local.set 5
                                  local.get 0
                                  local.get 6
                                  local.get 7
                                  select
                                  local.set 6
                                  local.get 1
                                  local.set 0
                                  local.get 1
                                  br_if 0 (;@14;)
                                end
                              end
                              local.get 6
                              call 5
                              local.get 5
                              i32.const 16
                              i32.lt_u
                              br_if 2 (;@10;)
                              local.get 6
                              local.get 2
                              i32.const 3
                              i32.or
                              i32.store offset=4
                              local.get 6
                              local.get 2
                              i32.add
                              local.tee 2
                              local.get 5
                              i32.const 1
                              i32.or
                              i32.store offset=4
                              local.get 2
                              local.get 5
                              i32.add
                              local.get 5
                              i32.store
                              i32.const 0
                              i32.load offset=1048992
                              local.tee 7
                              br_if 1 (;@11;)
                              br 5 (;@7;)
                            end
                            block ;; label = @12
                              block ;; label = @13
                                i32.const 2
                                local.get 1
                                i32.const 31
                                i32.and
                                local.tee 1
                                i32.shl
                                local.tee 5
                                i32.const 0
                                local.get 5
                                i32.sub
                                i32.or
                                local.get 0
                                local.get 1
                                i32.shl
                                i32.and
                                local.tee 0
                                i32.const 0
                                local.get 0
                                i32.sub
                                i32.and
                                i32.ctz
                                local.tee 1
                                i32.const 3
                                i32.shl
                                local.tee 6
                                i32.const 1048728
                                i32.add
                                i32.load
                                local.tee 0
                                i32.const 8
                                i32.add
                                local.tee 8
                                i32.load
                                local.tee 5
                                local.get 6
                                i32.const 1048720
                                i32.add
                                local.tee 6
                                i32.eq
                                br_if 0 (;@13;)
                                local.get 5
                                local.get 6
                                i32.store offset=12
                                local.get 6
                                local.get 5
                                i32.store offset=8
                                br 1 (;@12;)
                              end
                              i32.const 0
                              local.get 7
                              i32.const -2
                              local.get 1
                              i32.rotl
                              i32.and
                              i32.store offset=1048984
                            end
                            local.get 0
                            local.get 2
                            i32.const 3
                            i32.or
                            i32.store offset=4
                            local.get 0
                            local.get 2
                            i32.add
                            local.tee 7
                            local.get 1
                            i32.const 3
                            i32.shl
                            local.tee 1
                            local.get 2
                            i32.sub
                            local.tee 2
                            i32.const 1
                            i32.or
                            i32.store offset=4
                            local.get 0
                            local.get 1
                            i32.add
                            local.get 2
                            i32.store
                            i32.const 0
                            i32.load offset=1048992
                            local.tee 5
                            br_if 2 (;@9;)
                            br 3 (;@8;)
                          end
                          local.get 7
                          i32.const -8
                          i32.and
                          i32.const 1048720
                          i32.add
                          local.set 1
                          i32.const 0
                          i32.load offset=1049000
                          local.set 0
                          block ;; label = @11
                            block ;; label = @12
                              i32.const 0
                              i32.load offset=1048984
                              local.tee 8
                              i32.const 1
                              local.get 7
                              i32.const 3
                              i32.shr_u
                              i32.shl
                              local.tee 7
                              i32.and
                              i32.eqz
                              br_if 0 (;@12;)
                              local.get 1
                              i32.load offset=8
                              local.set 7
                              br 1 (;@11;)
                            end
                            i32.const 0
                            local.get 8
                            local.get 7
                            i32.or
                            i32.store offset=1048984
                            local.get 1
                            local.set 7
                          end
                          local.get 1
                          local.get 0
                          i32.store offset=8
                          local.get 7
                          local.get 0
                          i32.store offset=12
                          local.get 0
                          local.get 1
                          i32.store offset=12
                          local.get 0
                          local.get 7
                          i32.store offset=8
                          br 3 (;@7;)
                        end
                        local.get 6
                        local.get 5
                        local.get 2
                        i32.add
                        local.tee 0
                        i32.const 3
                        i32.or
                        i32.store offset=4
                        local.get 6
                        local.get 0
                        i32.add
                        local.tee 0
                        local.get 0
                        i32.load offset=4
                        i32.const 1
                        i32.or
                        i32.store offset=4
                        br 3 (;@6;)
                      end
                      local.get 5
                      i32.const -8
                      i32.and
                      i32.const 1048720
                      i32.add
                      local.set 1
                      i32.const 0
                      i32.load offset=1049000
                      local.set 0
                      block ;; label = @9
                        block ;; label = @10
                          i32.const 0
                          i32.load offset=1048984
                          local.tee 6
                          i32.const 1
                          local.get 5
                          i32.const 3
                          i32.shr_u
                          i32.shl
                          local.tee 5
                          i32.and
                          i32.eqz
                          br_if 0 (;@10;)
                          local.get 1
                          i32.load offset=8
                          local.set 5
                          br 1 (;@9;)
                        end
                        i32.const 0
                        local.get 6
                        local.get 5
                        i32.or
                        i32.store offset=1048984
                        local.get 1
                        local.set 5
                      end
                      local.get 1
                      local.get 0
                      i32.store offset=8
                      local.get 5
                      local.get 0
                      i32.store offset=12
                      local.get 0
                      local.get 1
                      i32.store offset=12
                      local.get 0
                      local.get 5
                      i32.store offset=8
                    end
                    i32.const 0
                    local.get 7
                    i32.store offset=1049000
                    i32.const 0
                    local.get 2
                    i32.store offset=1048992
                    local.get 8
                    return
                  end
                  i32.const 0
                  local.get 2
                  i32.store offset=1049000
                  i32.const 0
                  local.get 5
                  i32.store offset=1048992
                end
                local.get 6
                i32.const 8
                i32.add
                return
              end
              block ;; label = @5
                local.get 0
                local.get 6
                i32.or
                br_if 0 (;@5;)
                i32.const 0
                local.set 6
                local.get 3
                i32.const 2
                local.get 4
                i32.shl
                local.tee 0
                i32.const 0
                local.get 0
                i32.sub
                i32.or
                i32.and
                local.tee 0
                i32.eqz
                br_if 3 (;@2;)
                local.get 0
                i32.const 0
                local.get 0
                i32.sub
                i32.and
                i32.ctz
                i32.const 2
                i32.shl
                i32.const 1048576
                i32.add
                i32.load
                local.set 0
              end
              local.get 0
              i32.eqz
              br_if 1 (;@3;)
            end
            loop ;; label = @4
              local.get 0
              i32.load offset=4
              i32.const -8
              i32.and
              local.tee 5
              local.get 2
              i32.ge_u
              local.get 5
              local.get 2
              i32.sub
              local.tee 8
              local.get 1
              i32.lt_u
              i32.and
              local.set 7
              block ;; label = @5
                local.get 0
                i32.load offset=16
                local.tee 5
                br_if 0 (;@5;)
                local.get 0
                i32.const 20
                i32.add
                i32.load
                local.set 5
              end
              local.get 0
              local.get 6
              local.get 7
              select
              local.set 6
              local.get 8
              local.get 1
              local.get 7
              select
              local.set 1
              local.get 5
              local.set 0
              local.get 5
              br_if 0 (;@4;)
            end
          end
          local.get 6
          i32.eqz
          br_if 0 (;@2;)
          block ;; label = @3
            i32.const 0
            i32.load offset=1048992
            local.tee 0
            local.get 2
            i32.lt_u
            br_if 0 (;@3;)
            local.get 1
            local.get 0
            local.get 2
            i32.sub
            i32.ge_u
            br_if 1 (;@2;)
          end
          local.get 6
          call 5
          block ;; label = @3
            block ;; label = @4
              local.get 1
              i32.const 16
              i32.lt_u
              br_if 0 (;@4;)
              local.get 6
              local.get 2
              i32.const 3
              i32.or
              i32.store offset=4
              local.get 6
              local.get 2
              i32.add
              local.tee 0
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 1
              i32.add
              local.get 1
              i32.store
              block ;; label = @5
                local.get 1
                i32.const 256
                i32.lt_u
                br_if 0 (;@5;)
                local.get 0
                local.get 1
                call 6
                br 2 (;@3;)
              end
              local.get 1
              i32.const -8
              i32.and
              i32.const 1048720
              i32.add
              local.set 2
              block ;; label = @5
                block ;; label = @6
                  i32.const 0
                  i32.load offset=1048984
                  local.tee 5
                  i32.const 1
                  local.get 1
                  i32.const 3
                  i32.shr_u
                  i32.shl
                  local.tee 1
                  i32.and
                  i32.eqz
                  br_if 0 (;@6;)
                  local.get 2
                  i32.load offset=8
                  local.set 1
                  br 1 (;@5;)
                end
                i32.const 0
                local.get 5
                local.get 1
                i32.or
                i32.store offset=1048984
                local.get 2
                local.set 1
              end
              local.get 2
              local.get 0
              i32.store offset=8
              local.get 1
              local.get 0
              i32.store offset=12
              local.get 0
              local.get 2
              i32.store offset=12
              local.get 0
              local.get 1
              i32.store offset=8
              br 1 (;@3;)
            end
            local.get 6
            local.get 1
            local.get 2
            i32.add
            local.tee 0
            i32.const 3
            i32.or
            i32.store offset=4
            local.get 6
            local.get 0
            i32.add
            local.tee 0
            local.get 0
            i32.load offset=4
            i32.const 1
            i32.or
            i32.store offset=4
          end
          local.get 6
          i32.const 8
          i32.add
          return
        end
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  block ;; label = @7
                    block ;; label = @8
                      block ;; label = @9
                        block ;; label = @10
                          block ;; label = @11
                            i32.const 0
                            i32.load offset=1048992
                            local.tee 0
                            local.get 2
                            i32.ge_u
                            br_if 0 (;@11;)
                            block ;; label = @12
                              i32.const 0
                              i32.load offset=1048996
                              local.tee 0
                              local.get 2
                              i32.gt_u
                              br_if 0 (;@12;)
                              i32.const 0
                              local.set 1
                              local.get 2
                              i32.const 65583
                              i32.add
                              local.tee 5
                              i32.const 16
                              i32.shr_u
                              memory.grow
                              local.tee 0
                              i32.const -1
                              i32.eq
                              local.tee 6
                              br_if 11 (;@1;)
                              local.get 0
                              i32.const 16
                              i32.shl
                              local.tee 7
                              i32.eqz
                              br_if 11 (;@1;)
                              i32.const 0
                              i32.const 0
                              i32.load offset=1049008
                              i32.const 0
                              local.get 5
                              i32.const -65536
                              i32.and
                              local.get 6
                              select
                              local.tee 8
                              i32.add
                              local.tee 0
                              i32.store offset=1049008
                              i32.const 0
                              i32.const 0
                              i32.load offset=1049012
                              local.tee 1
                              local.get 0
                              local.get 1
                              local.get 0
                              i32.gt_u
                              select
                              i32.store offset=1049012
                              block ;; label = @13
                                block ;; label = @14
                                  block ;; label = @15
                                    i32.const 0
                                    i32.load offset=1049004
                                    local.tee 1
                                    i32.eqz
                                    br_if 0 (;@15;)
                                    i32.const 1048704
                                    local.set 0
                                    loop ;; label = @16
                                      local.get 0
                                      i32.load
                                      local.tee 5
                                      local.get 0
                                      i32.load offset=4
                                      local.tee 6
                                      i32.add
                                      local.get 7
                                      i32.eq
                                      br_if 2 (;@14;)
                                      local.get 0
                                      i32.load offset=8
                                      local.tee 0
                                      br_if 0 (;@16;)
                                      br 3 (;@13;)
                                    end
                                  end
                                  i32.const 0
                                  i32.load offset=1049020
                                  local.tee 0
                                  i32.eqz
                                  br_if 4 (;@10;)
                                  local.get 0
                                  local.get 7
                                  i32.gt_u
                                  br_if 4 (;@10;)
                                  br 11 (;@3;)
                                end
                                local.get 0
                                i32.load offset=12
                                br_if 0 (;@13;)
                                local.get 5
                                local.get 1
                                i32.gt_u
                                br_if 0 (;@13;)
                                local.get 1
                                local.get 7
                                i32.lt_u
                                br_if 4 (;@9;)
                              end
                              i32.const 0
                              i32.const 0
                              i32.load offset=1049020
                              local.tee 0
                              local.get 7
                              local.get 0
                              local.get 7
                              i32.lt_u
                              select
                              i32.store offset=1049020
                              local.get 7
                              local.get 8
                              i32.add
                              local.set 5
                              i32.const 1048704
                              local.set 0
                              block ;; label = @13
                                block ;; label = @14
                                  block ;; label = @15
                                    loop ;; label = @16
                                      local.get 0
                                      i32.load
                                      local.get 5
                                      i32.eq
                                      br_if 1 (;@15;)
                                      local.get 0
                                      i32.load offset=8
                                      local.tee 0
                                      br_if 0 (;@16;)
                                      br 2 (;@14;)
                                    end
                                  end
                                  local.get 0
                                  i32.load offset=12
                                  i32.eqz
                                  br_if 1 (;@13;)
                                end
                                i32.const 1048704
                                local.set 0
                                block ;; label = @14
                                  loop ;; label = @15
                                    block ;; label = @16
                                      local.get 0
                                      i32.load
                                      local.tee 5
                                      local.get 1
                                      i32.gt_u
                                      br_if 0 (;@16;)
                                      local.get 5
                                      local.get 0
                                      i32.load offset=4
                                      i32.add
                                      local.tee 5
                                      local.get 1
                                      i32.gt_u
                                      br_if 2 (;@14;)
                                    end
                                    local.get 0
                                    i32.load offset=8
                                    local.set 0
                                    br 0 (;@15;)
                                  end
                                end
                                i32.const 0
                                local.get 7
                                i32.store offset=1049004
                                i32.const 0
                                local.get 8
                                i32.const -40
                                i32.add
                                local.tee 0
                                i32.store offset=1048996
                                local.get 7
                                local.get 0
                                i32.const 1
                                i32.or
                                i32.store offset=4
                                local.get 7
                                local.get 0
                                i32.add
                                i32.const 40
                                i32.store offset=4
                                i32.const 0
                                i32.const 2097152
                                i32.store offset=1049016
                                local.get 1
                                local.get 5
                                i32.const -32
                                i32.add
                                i32.const -8
                                i32.and
                                i32.const -8
                                i32.add
                                local.tee 0
                                local.get 0
                                local.get 1
                                i32.const 16
                                i32.add
                                i32.lt_u
                                select
                                local.tee 6
                                i32.const 27
                                i32.store offset=4
                                i32.const 0
                                i64.load offset=1048704 align=4
                                local.set 9
                                local.get 6
                                i32.const 16
                                i32.add
                                i32.const 0
                                i64.load offset=1048712 align=4
                                i64.store align=4
                                local.get 6
                                local.get 9
                                i64.store offset=8 align=4
                                i32.const 0
                                local.get 8
                                i32.store offset=1048708
                                i32.const 0
                                local.get 7
                                i32.store offset=1048704
                                i32.const 0
                                local.get 6
                                i32.const 8
                                i32.add
                                i32.store offset=1048712
                                i32.const 0
                                i32.const 0
                                i32.store offset=1048716
                                local.get 6
                                i32.const 28
                                i32.add
                                local.set 0
                                loop ;; label = @14
                                  local.get 0
                                  i32.const 7
                                  i32.store
                                  local.get 0
                                  i32.const 4
                                  i32.add
                                  local.tee 0
                                  local.get 5
                                  i32.lt_u
                                  br_if 0 (;@14;)
                                end
                                local.get 6
                                local.get 1
                                i32.eq
                                br_if 11 (;@2;)
                                local.get 6
                                local.get 6
                                i32.load offset=4
                                i32.const -2
                                i32.and
                                i32.store offset=4
                                local.get 1
                                local.get 6
                                local.get 1
                                i32.sub
                                local.tee 0
                                i32.const 1
                                i32.or
                                i32.store offset=4
                                local.get 6
                                local.get 0
                                i32.store
                                block ;; label = @14
                                  local.get 0
                                  i32.const 256
                                  i32.lt_u
                                  br_if 0 (;@14;)
                                  local.get 1
                                  local.get 0
                                  call 6
                                  br 12 (;@2;)
                                end
                                local.get 0
                                i32.const -8
                                i32.and
                                i32.const 1048720
                                i32.add
                                local.set 5
                                block ;; label = @14
                                  block ;; label = @15
                                    i32.const 0
                                    i32.load offset=1048984
                                    local.tee 7
                                    i32.const 1
                                    local.get 0
                                    i32.const 3
                                    i32.shr_u
                                    i32.shl
                                    local.tee 0
                                    i32.and
                                    i32.eqz
                                    br_if 0 (;@15;)
                                    local.get 5
                                    i32.load offset=8
                                    local.set 0
                                    br 1 (;@14;)
                                  end
                                  i32.const 0
                                  local.get 7
                                  local.get 0
                                  i32.or
                                  i32.store offset=1048984
                                  local.get 5
                                  local.set 0
                                end
                                local.get 5
                                local.get 1
                                i32.store offset=8
                                local.get 0
                                local.get 1
                                i32.store offset=12
                                local.get 1
                                local.get 5
                                i32.store offset=12
                                local.get 1
                                local.get 0
                                i32.store offset=8
                                br 11 (;@2;)
                              end
                              local.get 0
                              local.get 7
                              i32.store
                              local.get 0
                              local.get 0
                              i32.load offset=4
                              local.get 8
                              i32.add
                              i32.store offset=4
                              local.get 7
                              local.get 2
                              i32.const 3
                              i32.or
                              i32.store offset=4
                              local.get 5
                              local.get 7
                              local.get 2
                              i32.add
                              local.tee 0
                              i32.sub
                              local.set 2
                              block ;; label = @13
                                local.get 5
                                i32.const 0
                                i32.load offset=1049004
                                i32.eq
                                br_if 0 (;@13;)
                                local.get 5
                                i32.const 0
                                i32.load offset=1049000
                                i32.eq
                                br_if 5 (;@8;)
                                local.get 5
                                i32.load offset=4
                                local.tee 1
                                i32.const 3
                                i32.and
                                i32.const 1
                                i32.ne
                                br_if 8 (;@5;)
                                block ;; label = @14
                                  block ;; label = @15
                                    local.get 1
                                    i32.const -8
                                    i32.and
                                    local.tee 6
                                    i32.const 256
                                    i32.lt_u
                                    br_if 0 (;@15;)
                                    local.get 5
                                    call 5
                                    br 1 (;@14;)
                                  end
                                  block ;; label = @15
                                    local.get 5
                                    i32.const 12
                                    i32.add
                                    i32.load
                                    local.tee 8
                                    local.get 5
                                    i32.const 8
                                    i32.add
                                    i32.load
                                    local.tee 4
                                    i32.eq
                                    br_if 0 (;@15;)
                                    local.get 4
                                    local.get 8
                                    i32.store offset=12
                                    local.get 8
                                    local.get 4
                                    i32.store offset=8
                                    br 1 (;@14;)
                                  end
                                  i32.const 0
                                  i32.const 0
                                  i32.load offset=1048984
                                  i32.const -2
                                  local.get 1
                                  i32.const 3
                                  i32.shr_u
                                  i32.rotl
                                  i32.and
                                  i32.store offset=1048984
                                end
                                local.get 6
                                local.get 2
                                i32.add
                                local.set 2
                                local.get 5
                                local.get 6
                                i32.add
                                local.tee 5
                                i32.load offset=4
                                local.set 1
                                br 8 (;@5;)
                              end
                              i32.const 0
                              local.get 0
                              i32.store offset=1049004
                              i32.const 0
                              i32.const 0
                              i32.load offset=1048996
                              local.get 2
                              i32.add
                              local.tee 2
                              i32.store offset=1048996
                              local.get 0
                              local.get 2
                              i32.const 1
                              i32.or
                              i32.store offset=4
                              br 8 (;@4;)
                            end
                            i32.const 0
                            local.get 0
                            local.get 2
                            i32.sub
                            local.tee 1
                            i32.store offset=1048996
                            i32.const 0
                            i32.const 0
                            i32.load offset=1049004
                            local.tee 0
                            local.get 2
                            i32.add
                            local.tee 5
                            i32.store offset=1049004
                            local.get 5
                            local.get 1
                            i32.const 1
                            i32.or
                            i32.store offset=4
                            local.get 0
                            local.get 2
                            i32.const 3
                            i32.or
                            i32.store offset=4
                            local.get 0
                            i32.const 8
                            i32.add
                            local.set 1
                            br 10 (;@1;)
                          end
                          i32.const 0
                          i32.load offset=1049000
                          local.set 1
                          local.get 0
                          local.get 2
                          i32.sub
                          local.tee 5
                          i32.const 16
                          i32.lt_u
                          br_if 3 (;@7;)
                          i32.const 0
                          local.get 5
                          i32.store offset=1048992
                          i32.const 0
                          local.get 1
                          local.get 2
                          i32.add
                          local.tee 7
                          i32.store offset=1049000
                          local.get 7
                          local.get 5
                          i32.const 1
                          i32.or
                          i32.store offset=4
                          local.get 1
                          local.get 0
                          i32.add
                          local.get 5
                          i32.store
                          local.get 1
                          local.get 2
                          i32.const 3
                          i32.or
                          i32.store offset=4
                          br 4 (;@6;)
                        end
                        i32.const 0
                        local.get 7
                        i32.store offset=1049020
                        br 6 (;@3;)
                      end
                      local.get 0
                      local.get 6
                      local.get 8
                      i32.add
                      i32.store offset=4
                      i32.const 0
                      i32.load offset=1049004
                      i32.const 0
                      i32.load offset=1048996
                      local.get 8
                      i32.add
                      call 7
                      br 6 (;@2;)
                    end
                    i32.const 0
                    local.get 0
                    i32.store offset=1049000
                    i32.const 0
                    i32.const 0
                    i32.load offset=1048992
                    local.get 2
                    i32.add
                    local.tee 2
                    i32.store offset=1048992
                    local.get 0
                    local.get 2
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    local.get 0
                    local.get 2
                    i32.add
                    local.get 2
                    i32.store
                    br 3 (;@4;)
                  end
                  i32.const 0
                  i32.const 0
                  i32.store offset=1049000
                  i32.const 0
                  i32.const 0
                  i32.store offset=1048992
                  local.get 1
                  local.get 0
                  i32.const 3
                  i32.or
                  i32.store offset=4
                  local.get 1
                  local.get 0
                  i32.add
                  local.tee 0
                  local.get 0
                  i32.load offset=4
                  i32.const 1
                  i32.or
                  i32.store offset=4
                end
                local.get 1
                i32.const 8
                i32.add
                return
              end
              local.get 5
              local.get 1
              i32.const -2
              i32.and
              i32.store offset=4
              local.get 0
              local.get 2
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 2
              i32.add
              local.get 2
              i32.store
              block ;; label = @5
                local.get 2
                i32.const 256
                i32.lt_u
                br_if 0 (;@5;)
                local.get 0
                local.get 2
                call 6
                br 1 (;@4;)
              end
              local.get 2
              i32.const -8
              i32.and
              i32.const 1048720
              i32.add
              local.set 1
              block ;; label = @5
                block ;; label = @6
                  i32.const 0
                  i32.load offset=1048984
                  local.tee 5
                  i32.const 1
                  local.get 2
                  i32.const 3
                  i32.shr_u
                  i32.shl
                  local.tee 2
                  i32.and
                  i32.eqz
                  br_if 0 (;@6;)
                  local.get 1
                  i32.load offset=8
                  local.set 2
                  br 1 (;@5;)
                end
                i32.const 0
                local.get 5
                local.get 2
                i32.or
                i32.store offset=1048984
                local.get 1
                local.set 2
              end
              local.get 1
              local.get 0
              i32.store offset=8
              local.get 2
              local.get 0
              i32.store offset=12
              local.get 0
              local.get 1
              i32.store offset=12
              local.get 0
              local.get 2
              i32.store offset=8
            end
            local.get 7
            i32.const 8
            i32.add
            return
          end
          i32.const 0
          i32.const 4095
          i32.store offset=1049024
          i32.const 0
          local.get 8
          i32.store offset=1048708
          i32.const 0
          local.get 7
          i32.store offset=1048704
          i32.const 0
          i32.const 1048720
          i32.store offset=1048732
          i32.const 0
          i32.const 1048728
          i32.store offset=1048740
          i32.const 0
          i32.const 1048720
          i32.store offset=1048728
          i32.const 0
          i32.const 1048736
          i32.store offset=1048748
          i32.const 0
          i32.const 1048728
          i32.store offset=1048736
          i32.const 0
          i32.const 1048744
          i32.store offset=1048756
          i32.const 0
          i32.const 1048736
          i32.store offset=1048744
          i32.const 0
          i32.const 1048752
          i32.store offset=1048764
          i32.const 0
          i32.const 1048744
          i32.store offset=1048752
          i32.const 0
          i32.const 1048760
          i32.store offset=1048772
          i32.const 0
          i32.const 1048752
          i32.store offset=1048760
          i32.const 0
          i32.const 1048768
          i32.store offset=1048780
          i32.const 0
          i32.const 1048760
          i32.store offset=1048768
          i32.const 0
          i32.const 1048776
          i32.store offset=1048788
          i32.const 0
          i32.const 1048768
          i32.store offset=1048776
          i32.const 0
          i32.const 0
          i32.store offset=1048716
          i32.const 0
          i32.const 1048784
          i32.store offset=1048796
          i32.const 0
          i32.const 1048776
          i32.store offset=1048784
          i32.const 0
          i32.const 1048784
          i32.store offset=1048792
          i32.const 0
          i32.const 1048792
          i32.store offset=1048804
          i32.const 0
          i32.const 1048792
          i32.store offset=1048800
          i32.const 0
          i32.const 1048800
          i32.store offset=1048812
          i32.const 0
          i32.const 1048800
          i32.store offset=1048808
          i32.const 0
          i32.const 1048808
          i32.store offset=1048820
          i32.const 0
          i32.const 1048808
          i32.store offset=1048816
          i32.const 0
          i32.const 1048816
          i32.store offset=1048828
          i32.const 0
          i32.const 1048816
          i32.store offset=1048824
          i32.const 0
          i32.const 1048824
          i32.store offset=1048836
          i32.const 0
          i32.const 1048824
          i32.store offset=1048832
          i32.const 0
          i32.const 1048832
          i32.store offset=1048844
          i32.const 0
          i32.const 1048832
          i32.store offset=1048840
          i32.const 0
          i32.const 1048840
          i32.store offset=1048852
          i32.const 0
          i32.const 1048840
          i32.store offset=1048848
          i32.const 0
          i32.const 1048848
          i32.store offset=1048860
          i32.const 0
          i32.const 1048856
          i32.store offset=1048868
          i32.const 0
          i32.const 1048848
          i32.store offset=1048856
          i32.const 0
          i32.const 1048864
          i32.store offset=1048876
          i32.const 0
          i32.const 1048856
          i32.store offset=1048864
          i32.const 0
          i32.const 1048872
          i32.store offset=1048884
          i32.const 0
          i32.const 1048864
          i32.store offset=1048872
          i32.const 0
          i32.const 1048880
          i32.store offset=1048892
          i32.const 0
          i32.const 1048872
          i32.store offset=1048880
          i32.const 0
          i32.const 1048888
          i32.store offset=1048900
          i32.const 0
          i32.const 1048880
          i32.store offset=1048888
          i32.const 0
          i32.const 1048896
          i32.store offset=1048908
          i32.const 0
          i32.const 1048888
          i32.store offset=1048896
          i32.const 0
          i32.const 1048904
          i32.store offset=1048916
          i32.const 0
          i32.const 1048896
          i32.store offset=1048904
          i32.const 0
          i32.const 1048912
          i32.store offset=1048924
          i32.const 0
          i32.const 1048904
          i32.store offset=1048912
          i32.const 0
          i32.const 1048920
          i32.store offset=1048932
          i32.const 0
          i32.const 1048912
          i32.store offset=1048920
          i32.const 0
          i32.const 1048928
          i32.store offset=1048940
          i32.const 0
          i32.const 1048920
          i32.store offset=1048928
          i32.const 0
          i32.const 1048936
          i32.store offset=1048948
          i32.const 0
          i32.const 1048928
          i32.store offset=1048936
          i32.const 0
          i32.const 1048944
          i32.store offset=1048956
          i32.const 0
          i32.const 1048936
          i32.store offset=1048944
          i32.const 0
          i32.const 1048952
          i32.store offset=1048964
          i32.const 0
          i32.const 1048944
          i32.store offset=1048952
          i32.const 0
          i32.const 1048960
          i32.store offset=1048972
          i32.const 0
          i32.const 1048952
          i32.store offset=1048960
          i32.const 0
          i32.const 1048968
          i32.store offset=1048980
          i32.const 0
          i32.const 1048960
          i32.store offset=1048968
          i32.const 0
          local.get 7
          i32.store offset=1049004
          i32.const 0
          i32.const 1048968
          i32.store offset=1048976
          i32.const 0
          local.get 8
          i32.const -40
          i32.add
          local.tee 0
          i32.store offset=1048996
          local.get 7
          local.get 0
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 7
          local.get 0
          i32.add
          i32.const 40
          i32.store offset=4
          i32.const 0
          i32.const 2097152
          i32.store offset=1049016
        end
        i32.const 0
        local.set 1
        i32.const 0
        i32.load offset=1048996
        local.tee 0
        local.get 2
        i32.le_u
        br_if 0 (;@1;)
        i32.const 0
        local.get 0
        local.get 2
        i32.sub
        local.tee 1
        i32.store offset=1048996
        i32.const 0
        i32.const 0
        i32.load offset=1049004
        local.tee 0
        local.get 2
        i32.add
        local.tee 5
        i32.store offset=1049004
        local.get 5
        local.get 1
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 0
        local.get 2
        i32.const 3
        i32.or
        i32.store offset=4
        local.get 0
        i32.const 8
        i32.add
        return
      end
      local.get 1
    )
    (func (;4;) (type 3) (param i32 i32)
      (local i32 i32 i32 i32)
      local.get 0
      local.get 1
      i32.add
      local.set 2
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 0
            i32.load offset=4
            local.tee 3
            i32.const 1
            i32.and
            br_if 0 (;@3;)
            local.get 3
            i32.const 3
            i32.and
            i32.eqz
            br_if 1 (;@2;)
            local.get 0
            i32.load
            local.tee 3
            local.get 1
            i32.add
            local.set 1
            block ;; label = @4
              local.get 0
              local.get 3
              i32.sub
              local.tee 0
              i32.const 0
              i32.load offset=1049000
              i32.ne
              br_if 0 (;@4;)
              local.get 2
              i32.load offset=4
              i32.const 3
              i32.and
              i32.const 3
              i32.ne
              br_if 1 (;@3;)
              i32.const 0
              local.get 1
              i32.store offset=1048992
              local.get 2
              local.get 2
              i32.load offset=4
              i32.const -2
              i32.and
              i32.store offset=4
              local.get 0
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 2
              local.get 1
              i32.store
              return
            end
            block ;; label = @4
              local.get 3
              i32.const 256
              i32.lt_u
              br_if 0 (;@4;)
              local.get 0
              call 5
              br 1 (;@3;)
            end
            block ;; label = @4
              local.get 0
              i32.const 12
              i32.add
              i32.load
              local.tee 4
              local.get 0
              i32.const 8
              i32.add
              i32.load
              local.tee 5
              i32.eq
              br_if 0 (;@4;)
              local.get 5
              local.get 4
              i32.store offset=12
              local.get 4
              local.get 5
              i32.store offset=8
              br 1 (;@3;)
            end
            i32.const 0
            i32.const 0
            i32.load offset=1048984
            i32.const -2
            local.get 3
            i32.const 3
            i32.shr_u
            i32.rotl
            i32.and
            i32.store offset=1048984
          end
          block ;; label = @3
            local.get 2
            i32.load offset=4
            local.tee 3
            i32.const 2
            i32.and
            i32.eqz
            br_if 0 (;@3;)
            local.get 2
            local.get 3
            i32.const -2
            i32.and
            i32.store offset=4
            local.get 0
            local.get 1
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 0
            local.get 1
            i32.add
            local.get 1
            i32.store
            br 2 (;@1;)
          end
          block ;; label = @3
            block ;; label = @4
              local.get 2
              i32.const 0
              i32.load offset=1049004
              i32.eq
              br_if 0 (;@4;)
              local.get 2
              i32.const 0
              i32.load offset=1049000
              i32.eq
              br_if 1 (;@3;)
              local.get 3
              i32.const -8
              i32.and
              local.tee 4
              local.get 1
              i32.add
              local.set 1
              block ;; label = @5
                block ;; label = @6
                  local.get 4
                  i32.const 256
                  i32.lt_u
                  br_if 0 (;@6;)
                  local.get 2
                  call 5
                  br 1 (;@5;)
                end
                block ;; label = @6
                  local.get 2
                  i32.const 12
                  i32.add
                  i32.load
                  local.tee 4
                  local.get 2
                  i32.const 8
                  i32.add
                  i32.load
                  local.tee 2
                  i32.eq
                  br_if 0 (;@6;)
                  local.get 2
                  local.get 4
                  i32.store offset=12
                  local.get 4
                  local.get 2
                  i32.store offset=8
                  br 1 (;@5;)
                end
                i32.const 0
                i32.const 0
                i32.load offset=1048984
                i32.const -2
                local.get 3
                i32.const 3
                i32.shr_u
                i32.rotl
                i32.and
                i32.store offset=1048984
              end
              local.get 0
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 1
              i32.add
              local.get 1
              i32.store
              local.get 0
              i32.const 0
              i32.load offset=1049000
              i32.ne
              br_if 3 (;@1;)
              i32.const 0
              local.get 1
              i32.store offset=1048992
              br 2 (;@2;)
            end
            i32.const 0
            local.get 0
            i32.store offset=1049004
            i32.const 0
            i32.const 0
            i32.load offset=1048996
            local.get 1
            i32.add
            local.tee 1
            i32.store offset=1048996
            local.get 0
            local.get 1
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 0
            i32.const 0
            i32.load offset=1049000
            i32.ne
            br_if 1 (;@2;)
            i32.const 0
            i32.const 0
            i32.store offset=1048992
            i32.const 0
            i32.const 0
            i32.store offset=1049000
            return
          end
          i32.const 0
          local.get 0
          i32.store offset=1049000
          i32.const 0
          i32.const 0
          i32.load offset=1048992
          local.get 1
          i32.add
          local.tee 1
          i32.store offset=1048992
          local.get 0
          local.get 1
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 0
          local.get 1
          i32.add
          local.get 1
          i32.store
          return
        end
        return
      end
      block ;; label = @1
        local.get 1
        i32.const 256
        i32.lt_u
        br_if 0 (;@1;)
        local.get 0
        local.get 1
        call 6
        return
      end
      local.get 1
      i32.const -8
      i32.and
      i32.const 1048720
      i32.add
      local.set 2
      block ;; label = @1
        block ;; label = @2
          i32.const 0
          i32.load offset=1048984
          local.tee 3
          i32.const 1
          local.get 1
          i32.const 3
          i32.shr_u
          i32.shl
          local.tee 1
          i32.and
          i32.eqz
          br_if 0 (;@2;)
          local.get 2
          i32.load offset=8
          local.set 1
          br 1 (;@1;)
        end
        i32.const 0
        local.get 3
        local.get 1
        i32.or
        i32.store offset=1048984
        local.get 2
        local.set 1
      end
      local.get 2
      local.get 0
      i32.store offset=8
      local.get 1
      local.get 0
      i32.store offset=12
      local.get 0
      local.get 2
      i32.store offset=12
      local.get 0
      local.get 1
      i32.store offset=8
    )
    (func (;5;) (type 4) (param i32)
      (local i32 i32 i32 i32 i32)
      local.get 0
      i32.load offset=24
      local.set 1
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 0
            i32.load offset=12
            local.tee 2
            local.get 0
            i32.ne
            br_if 0 (;@3;)
            local.get 0
            i32.const 20
            i32.const 16
            local.get 0
            i32.const 20
            i32.add
            local.tee 2
            i32.load
            local.tee 3
            select
            i32.add
            i32.load
            local.tee 4
            br_if 1 (;@2;)
            i32.const 0
            local.set 2
            br 2 (;@1;)
          end
          local.get 0
          i32.load offset=8
          local.tee 4
          local.get 2
          i32.store offset=12
          local.get 2
          local.get 4
          i32.store offset=8
          br 1 (;@1;)
        end
        local.get 2
        local.get 0
        i32.const 16
        i32.add
        local.get 3
        select
        local.set 3
        loop ;; label = @2
          local.get 3
          local.set 5
          local.get 4
          local.tee 2
          i32.const 20
          i32.add
          local.tee 4
          local.get 2
          i32.const 16
          i32.add
          local.get 4
          i32.load
          local.tee 4
          select
          local.set 3
          local.get 2
          i32.const 20
          i32.const 16
          local.get 4
          select
          i32.add
          i32.load
          local.tee 4
          br_if 0 (;@2;)
        end
        local.get 5
        i32.const 0
        i32.store
      end
      block ;; label = @1
        local.get 1
        i32.eqz
        br_if 0 (;@1;)
        block ;; label = @2
          block ;; label = @3
            local.get 0
            i32.load offset=28
            i32.const 2
            i32.shl
            i32.const 1048576
            i32.add
            local.tee 4
            i32.load
            local.get 0
            i32.eq
            br_if 0 (;@3;)
            local.get 1
            i32.const 16
            i32.const 20
            local.get 1
            i32.load offset=16
            local.get 0
            i32.eq
            select
            i32.add
            local.get 2
            i32.store
            local.get 2
            br_if 1 (;@2;)
            br 2 (;@1;)
          end
          local.get 4
          local.get 2
          i32.store
          local.get 2
          br_if 0 (;@2;)
          i32.const 0
          i32.const 0
          i32.load offset=1048988
          i32.const -2
          local.get 0
          i32.load offset=28
          i32.rotl
          i32.and
          i32.store offset=1048988
          return
        end
        local.get 2
        local.get 1
        i32.store offset=24
        block ;; label = @2
          local.get 0
          i32.load offset=16
          local.tee 4
          i32.eqz
          br_if 0 (;@2;)
          local.get 2
          local.get 4
          i32.store offset=16
          local.get 4
          local.get 2
          i32.store offset=24
        end
        local.get 0
        i32.const 20
        i32.add
        i32.load
        local.tee 4
        i32.eqz
        br_if 0 (;@1;)
        local.get 2
        i32.const 20
        i32.add
        local.get 4
        i32.store
        local.get 4
        local.get 2
        i32.store offset=24
        return
      end
    )
    (func (;6;) (type 3) (param i32 i32)
      (local i32 i32 i32 i32)
      i32.const 31
      local.set 2
      block ;; label = @1
        local.get 1
        i32.const 16777215
        i32.gt_u
        br_if 0 (;@1;)
        local.get 1
        i32.const 6
        local.get 1
        i32.const 8
        i32.shr_u
        i32.clz
        local.tee 2
        i32.sub
        i32.shr_u
        i32.const 1
        i32.and
        local.get 2
        i32.const 1
        i32.shl
        i32.sub
        i32.const 62
        i32.add
        local.set 2
      end
      local.get 0
      i64.const 0
      i64.store offset=16 align=4
      local.get 0
      local.get 2
      i32.store offset=28
      local.get 2
      i32.const 2
      i32.shl
      i32.const 1048576
      i32.add
      local.set 3
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                i32.const 0
                i32.load offset=1048988
                local.tee 4
                i32.const 1
                local.get 2
                i32.shl
                local.tee 5
                i32.and
                i32.eqz
                br_if 0 (;@5;)
                local.get 3
                i32.load
                local.tee 4
                i32.load offset=4
                i32.const -8
                i32.and
                local.get 1
                i32.ne
                br_if 1 (;@4;)
                local.get 4
                local.set 2
                br 2 (;@3;)
              end
              i32.const 0
              local.get 4
              local.get 5
              i32.or
              i32.store offset=1048988
              local.get 3
              local.get 0
              i32.store
              local.get 0
              local.get 3
              i32.store offset=24
              br 3 (;@1;)
            end
            local.get 1
            i32.const 0
            i32.const 25
            local.get 2
            i32.const 1
            i32.shr_u
            i32.sub
            i32.const 31
            i32.and
            local.get 2
            i32.const 31
            i32.eq
            select
            i32.shl
            local.set 3
            loop ;; label = @4
              local.get 4
              local.get 3
              i32.const 29
              i32.shr_u
              i32.const 4
              i32.and
              i32.add
              i32.const 16
              i32.add
              local.tee 5
              i32.load
              local.tee 2
              i32.eqz
              br_if 2 (;@2;)
              local.get 3
              i32.const 1
              i32.shl
              local.set 3
              local.get 2
              local.set 4
              local.get 2
              i32.load offset=4
              i32.const -8
              i32.and
              local.get 1
              i32.ne
              br_if 0 (;@4;)
            end
          end
          local.get 2
          i32.load offset=8
          local.tee 3
          local.get 0
          i32.store offset=12
          local.get 2
          local.get 0
          i32.store offset=8
          local.get 0
          i32.const 0
          i32.store offset=24
          local.get 0
          local.get 2
          i32.store offset=12
          local.get 0
          local.get 3
          i32.store offset=8
          return
        end
        local.get 5
        local.get 0
        i32.store
        local.get 0
        local.get 4
        i32.store offset=24
      end
      local.get 0
      local.get 0
      i32.store offset=12
      local.get 0
      local.get 0
      i32.store offset=8
    )
    (func (;7;) (type 3) (param i32 i32)
      (local i32 i32)
      i32.const 0
      local.get 0
      i32.const 15
      i32.add
      i32.const -8
      i32.and
      local.tee 2
      i32.const -8
      i32.add
      i32.store offset=1049004
      i32.const 0
      local.get 0
      local.get 2
      i32.sub
      local.get 1
      i32.add
      i32.const 8
      i32.add
      local.tee 3
      i32.store offset=1048996
      local.get 2
      i32.const -4
      i32.add
      local.get 3
      i32.const 1
      i32.or
      i32.store
      local.get 0
      local.get 1
      i32.add
      i32.const 40
      i32.store offset=4
      i32.const 0
      i32.const 2097152
      i32.store offset=1049016
    )
    (func (;8;) (type 4) (param i32)
      (local i32 i32 i32 i32 i32)
      local.get 0
      i32.const -8
      i32.add
      local.tee 1
      local.get 0
      i32.const -4
      i32.add
      i32.load
      local.tee 2
      i32.const -8
      i32.and
      local.tee 0
      i32.add
      local.set 3
      block ;; label = @1
        block ;; label = @2
          local.get 2
          i32.const 1
          i32.and
          br_if 0 (;@2;)
          local.get 2
          i32.const 3
          i32.and
          i32.eqz
          br_if 1 (;@1;)
          local.get 1
          i32.load
          local.tee 2
          local.get 0
          i32.add
          local.set 0
          block ;; label = @3
            local.get 1
            local.get 2
            i32.sub
            local.tee 1
            i32.const 0
            i32.load offset=1049000
            i32.ne
            br_if 0 (;@3;)
            local.get 3
            i32.load offset=4
            i32.const 3
            i32.and
            i32.const 3
            i32.ne
            br_if 1 (;@2;)
            i32.const 0
            local.get 0
            i32.store offset=1048992
            local.get 3
            local.get 3
            i32.load offset=4
            i32.const -2
            i32.and
            i32.store offset=4
            local.get 1
            local.get 0
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 1
            local.get 0
            i32.add
            local.get 0
            i32.store
            return
          end
          block ;; label = @3
            local.get 2
            i32.const 256
            i32.lt_u
            br_if 0 (;@3;)
            local.get 1
            call 5
            br 1 (;@2;)
          end
          block ;; label = @3
            local.get 1
            i32.const 12
            i32.add
            i32.load
            local.tee 4
            local.get 1
            i32.const 8
            i32.add
            i32.load
            local.tee 5
            i32.eq
            br_if 0 (;@3;)
            local.get 5
            local.get 4
            i32.store offset=12
            local.get 4
            local.get 5
            i32.store offset=8
            br 1 (;@2;)
          end
          i32.const 0
          i32.const 0
          i32.load offset=1048984
          i32.const -2
          local.get 2
          i32.const 3
          i32.shr_u
          i32.rotl
          i32.and
          i32.store offset=1048984
        end
        block ;; label = @2
          block ;; label = @3
            local.get 3
            i32.load offset=4
            local.tee 2
            i32.const 2
            i32.and
            i32.eqz
            br_if 0 (;@3;)
            local.get 3
            local.get 2
            i32.const -2
            i32.and
            i32.store offset=4
            local.get 1
            local.get 0
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 1
            local.get 0
            i32.add
            local.get 0
            i32.store
            br 1 (;@2;)
          end
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  local.get 3
                  i32.const 0
                  i32.load offset=1049004
                  i32.eq
                  br_if 0 (;@6;)
                  local.get 3
                  i32.const 0
                  i32.load offset=1049000
                  i32.eq
                  br_if 1 (;@5;)
                  local.get 2
                  i32.const -8
                  i32.and
                  local.tee 4
                  local.get 0
                  i32.add
                  local.set 0
                  block ;; label = @7
                    block ;; label = @8
                      local.get 4
                      i32.const 256
                      i32.lt_u
                      br_if 0 (;@8;)
                      local.get 3
                      call 5
                      br 1 (;@7;)
                    end
                    block ;; label = @8
                      local.get 3
                      i32.const 12
                      i32.add
                      i32.load
                      local.tee 4
                      local.get 3
                      i32.const 8
                      i32.add
                      i32.load
                      local.tee 3
                      i32.eq
                      br_if 0 (;@8;)
                      local.get 3
                      local.get 4
                      i32.store offset=12
                      local.get 4
                      local.get 3
                      i32.store offset=8
                      br 1 (;@7;)
                    end
                    i32.const 0
                    i32.const 0
                    i32.load offset=1048984
                    i32.const -2
                    local.get 2
                    i32.const 3
                    i32.shr_u
                    i32.rotl
                    i32.and
                    i32.store offset=1048984
                  end
                  local.get 1
                  local.get 0
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 1
                  local.get 0
                  i32.add
                  local.get 0
                  i32.store
                  local.get 1
                  i32.const 0
                  i32.load offset=1049000
                  i32.ne
                  br_if 4 (;@2;)
                  i32.const 0
                  local.get 0
                  i32.store offset=1048992
                  return
                end
                i32.const 0
                local.get 1
                i32.store offset=1049004
                i32.const 0
                i32.const 0
                i32.load offset=1048996
                local.get 0
                i32.add
                local.tee 0
                i32.store offset=1048996
                local.get 1
                local.get 0
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 1
                i32.const 0
                i32.load offset=1049000
                i32.eq
                br_if 1 (;@4;)
                br 2 (;@3;)
              end
              i32.const 0
              local.get 1
              i32.store offset=1049000
              i32.const 0
              i32.const 0
              i32.load offset=1048992
              local.get 0
              i32.add
              local.tee 0
              i32.store offset=1048992
              local.get 1
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 1
              local.get 0
              i32.add
              local.get 0
              i32.store
              return
            end
            i32.const 0
            i32.const 0
            i32.store offset=1048992
            i32.const 0
            i32.const 0
            i32.store offset=1049000
          end
          local.get 0
          i32.const 0
          i32.load offset=1049016
          i32.le_u
          br_if 1 (;@1;)
          i32.const 0
          i32.load offset=1049004
          local.tee 0
          i32.eqz
          br_if 1 (;@1;)
          block ;; label = @3
            i32.const 0
            i32.load offset=1048996
            i32.const 41
            i32.lt_u
            br_if 0 (;@3;)
            i32.const 1048704
            local.set 1
            loop ;; label = @4
              block ;; label = @5
                local.get 1
                i32.load
                local.tee 3
                local.get 0
                i32.gt_u
                br_if 0 (;@5;)
                local.get 3
                local.get 1
                i32.load offset=4
                i32.add
                local.get 0
                i32.gt_u
                br_if 2 (;@3;)
              end
              local.get 1
              i32.load offset=8
              local.tee 1
              br_if 0 (;@4;)
            end
          end
          call 9
          i32.const 0
          i32.load offset=1048996
          i32.const 0
          i32.load offset=1049016
          i32.le_u
          br_if 1 (;@1;)
          i32.const 0
          i32.const -1
          i32.store offset=1049016
          return
        end
        block ;; label = @2
          local.get 0
          i32.const 256
          i32.lt_u
          br_if 0 (;@2;)
          local.get 1
          local.get 0
          call 6
          i32.const 0
          i32.const 0
          i32.load offset=1049024
          i32.const -1
          i32.add
          local.tee 1
          i32.store offset=1049024
          local.get 1
          br_if 1 (;@1;)
          call 9
          return
        end
        local.get 0
        i32.const -8
        i32.and
        i32.const 1048720
        i32.add
        local.set 3
        block ;; label = @2
          block ;; label = @3
            i32.const 0
            i32.load offset=1048984
            local.tee 2
            i32.const 1
            local.get 0
            i32.const 3
            i32.shr_u
            i32.shl
            local.tee 0
            i32.and
            i32.eqz
            br_if 0 (;@3;)
            local.get 3
            i32.load offset=8
            local.set 0
            br 1 (;@2;)
          end
          i32.const 0
          local.get 2
          local.get 0
          i32.or
          i32.store offset=1048984
          local.get 3
          local.set 0
        end
        local.get 3
        local.get 1
        i32.store offset=8
        local.get 0
        local.get 1
        i32.store offset=12
        local.get 1
        local.get 3
        i32.store offset=12
        local.get 1
        local.get 0
        i32.store offset=8
      end
    )
    (func (;9;) (type 0)
      (local i32 i32)
      i32.const 0
      local.set 0
      block ;; label = @1
        i32.const 0
        i32.load offset=1048712
        local.tee 1
        i32.eqz
        br_if 0 (;@1;)
        i32.const 0
        local.set 0
        loop ;; label = @2
          local.get 0
          i32.const 1
          i32.add
          local.set 0
          local.get 1
          i32.load offset=8
          local.tee 1
          br_if 0 (;@2;)
        end
      end
      i32.const 0
      local.get 0
      i32.const 4095
      local.get 0
      i32.const 4095
      i32.gt_u
      select
      i32.store offset=1049024
    )
    (func (;10;) (type 5) (param i32 i32 i32 i32) (result i32)
      (local i32 i32 i32 i32 i32)
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  block ;; label = @7
                    block ;; label = @8
                      local.get 1
                      br_if 0 (;@8;)
                      local.get 3
                      br_if 1 (;@7;)
                      local.get 2
                      return
                    end
                    block ;; label = @8
                      local.get 2
                      i32.const 9
                      i32.lt_u
                      br_if 0 (;@8;)
                      local.get 2
                      local.get 3
                      call 2
                      local.tee 2
                      i32.eqz
                      br_if 5 (;@3;)
                      local.get 2
                      local.get 0
                      local.get 1
                      local.get 3
                      local.get 1
                      local.get 3
                      i32.lt_u
                      select
                      call 12
                      local.set 3
                      local.get 0
                      call 8
                      local.get 3
                      return
                    end
                    local.get 3
                    i32.const -65588
                    i32.gt_u
                    br_if 4 (;@3;)
                    i32.const 16
                    local.get 3
                    i32.const 11
                    i32.add
                    i32.const -8
                    i32.and
                    local.get 3
                    i32.const 11
                    i32.lt_u
                    select
                    local.set 1
                    local.get 0
                    i32.const -4
                    i32.add
                    local.tee 4
                    i32.load
                    local.tee 5
                    i32.const -8
                    i32.and
                    local.set 2
                    block ;; label = @8
                      block ;; label = @9
                        block ;; label = @10
                          block ;; label = @11
                            block ;; label = @12
                              block ;; label = @13
                                block ;; label = @14
                                  local.get 5
                                  i32.const 3
                                  i32.and
                                  i32.eqz
                                  br_if 0 (;@14;)
                                  local.get 0
                                  i32.const -8
                                  i32.add
                                  local.set 6
                                  local.get 2
                                  local.get 1
                                  i32.ge_u
                                  br_if 1 (;@13;)
                                  local.get 6
                                  local.get 2
                                  i32.add
                                  local.tee 7
                                  i32.const 0
                                  i32.load offset=1049004
                                  i32.eq
                                  br_if 5 (;@9;)
                                  local.get 7
                                  i32.const 0
                                  i32.load offset=1049000
                                  i32.eq
                                  br_if 4 (;@10;)
                                  local.get 7
                                  i32.load offset=4
                                  local.tee 5
                                  i32.const 2
                                  i32.and
                                  br_if 6 (;@8;)
                                  local.get 5
                                  i32.const -8
                                  i32.and
                                  local.tee 8
                                  local.get 2
                                  i32.add
                                  local.tee 2
                                  local.get 1
                                  i32.lt_u
                                  br_if 6 (;@8;)
                                  local.get 2
                                  local.get 1
                                  i32.sub
                                  local.set 3
                                  local.get 8
                                  i32.const 256
                                  i32.lt_u
                                  br_if 2 (;@12;)
                                  local.get 7
                                  call 5
                                  br 3 (;@11;)
                                end
                                local.get 1
                                i32.const 256
                                i32.lt_u
                                br_if 5 (;@8;)
                                local.get 2
                                local.get 1
                                i32.const 4
                                i32.or
                                i32.lt_u
                                br_if 5 (;@8;)
                                local.get 2
                                local.get 1
                                i32.sub
                                i32.const 131073
                                i32.ge_u
                                br_if 5 (;@8;)
                                br 11 (;@2;)
                              end
                              local.get 2
                              local.get 1
                              i32.sub
                              local.tee 3
                              i32.const 16
                              i32.lt_u
                              br_if 10 (;@2;)
                              local.get 4
                              local.get 5
                              i32.const 1
                              i32.and
                              local.get 1
                              i32.or
                              i32.const 2
                              i32.or
                              i32.store
                              local.get 6
                              local.get 1
                              i32.add
                              local.tee 1
                              local.get 3
                              i32.const 3
                              i32.or
                              i32.store offset=4
                              local.get 1
                              local.get 3
                              i32.add
                              local.tee 2
                              local.get 2
                              i32.load offset=4
                              i32.const 1
                              i32.or
                              i32.store offset=4
                              local.get 1
                              local.get 3
                              call 4
                              local.get 0
                              return
                            end
                            block ;; label = @12
                              local.get 7
                              i32.const 12
                              i32.add
                              i32.load
                              local.tee 8
                              local.get 7
                              i32.const 8
                              i32.add
                              i32.load
                              local.tee 7
                              i32.eq
                              br_if 0 (;@12;)
                              local.get 7
                              local.get 8
                              i32.store offset=12
                              local.get 8
                              local.get 7
                              i32.store offset=8
                              br 1 (;@11;)
                            end
                            i32.const 0
                            i32.const 0
                            i32.load offset=1048984
                            i32.const -2
                            local.get 5
                            i32.const 3
                            i32.shr_u
                            i32.rotl
                            i32.and
                            i32.store offset=1048984
                          end
                          local.get 3
                          i32.const 16
                          i32.lt_u
                          br_if 4 (;@6;)
                          local.get 4
                          local.get 4
                          i32.load
                          i32.const 1
                          i32.and
                          local.get 1
                          i32.or
                          i32.const 2
                          i32.or
                          i32.store
                          local.get 6
                          local.get 1
                          i32.add
                          local.tee 1
                          local.get 3
                          i32.const 3
                          i32.or
                          i32.store offset=4
                          local.get 1
                          local.get 3
                          i32.add
                          local.tee 2
                          local.get 2
                          i32.load offset=4
                          i32.const 1
                          i32.or
                          i32.store offset=4
                          local.get 1
                          local.get 3
                          call 4
                          local.get 0
                          return
                        end
                        i32.const 0
                        i32.load offset=1048992
                        local.get 2
                        i32.add
                        local.tee 2
                        local.get 1
                        i32.lt_u
                        br_if 1 (;@8;)
                        block ;; label = @10
                          block ;; label = @11
                            local.get 2
                            local.get 1
                            i32.sub
                            local.tee 3
                            i32.const 15
                            i32.gt_u
                            br_if 0 (;@11;)
                            local.get 4
                            local.get 5
                            i32.const 1
                            i32.and
                            local.get 2
                            i32.or
                            i32.const 2
                            i32.or
                            i32.store
                            local.get 6
                            local.get 2
                            i32.add
                            local.tee 3
                            local.get 3
                            i32.load offset=4
                            i32.const 1
                            i32.or
                            i32.store offset=4
                            i32.const 0
                            local.set 3
                            i32.const 0
                            local.set 1
                            br 1 (;@10;)
                          end
                          local.get 4
                          local.get 5
                          i32.const 1
                          i32.and
                          local.get 1
                          i32.or
                          i32.const 2
                          i32.or
                          i32.store
                          local.get 6
                          local.get 1
                          i32.add
                          local.tee 1
                          local.get 3
                          i32.const 1
                          i32.or
                          i32.store offset=4
                          local.get 1
                          local.get 3
                          i32.add
                          local.tee 2
                          local.get 3
                          i32.store
                          local.get 2
                          local.get 2
                          i32.load offset=4
                          i32.const -2
                          i32.and
                          i32.store offset=4
                        end
                        i32.const 0
                        local.get 1
                        i32.store offset=1049000
                        i32.const 0
                        local.get 3
                        i32.store offset=1048992
                        local.get 0
                        return
                      end
                      i32.const 0
                      i32.load offset=1048996
                      local.get 2
                      i32.add
                      local.tee 2
                      local.get 1
                      i32.gt_u
                      br_if 7 (;@1;)
                    end
                    local.get 3
                    call 3
                    local.tee 1
                    i32.eqz
                    br_if 4 (;@3;)
                    local.get 1
                    local.get 0
                    i32.const -4
                    i32.const -8
                    local.get 4
                    i32.load
                    local.tee 2
                    i32.const 3
                    i32.and
                    select
                    local.get 2
                    i32.const -8
                    i32.and
                    i32.add
                    local.tee 2
                    local.get 3
                    local.get 2
                    local.get 3
                    i32.lt_u
                    select
                    call 12
                    local.set 3
                    local.get 0
                    call 8
                    local.get 3
                    return
                  end
                  i32.const 0
                  i32.load8_u offset=1049028
                  drop
                  local.get 2
                  i32.const 9
                  i32.lt_u
                  br_if 1 (;@5;)
                  local.get 2
                  local.get 3
                  call 2
                  local.set 0
                  br 2 (;@4;)
                end
                local.get 4
                local.get 4
                i32.load
                i32.const 1
                i32.and
                local.get 2
                i32.or
                i32.const 2
                i32.or
                i32.store
                local.get 6
                local.get 2
                i32.add
                local.tee 3
                local.get 3
                i32.load offset=4
                i32.const 1
                i32.or
                i32.store offset=4
                br 3 (;@2;)
              end
              local.get 3
              call 3
              local.set 0
            end
            local.get 0
            br_if 1 (;@2;)
          end
          unreachable
          unreachable
        end
        local.get 0
        return
      end
      local.get 4
      local.get 5
      i32.const 1
      i32.and
      local.get 1
      i32.or
      i32.const 2
      i32.or
      i32.store
      local.get 6
      local.get 1
      i32.add
      local.tee 3
      local.get 2
      local.get 1
      i32.sub
      local.tee 1
      i32.const 1
      i32.or
      i32.store offset=4
      i32.const 0
      local.get 1
      i32.store offset=1048996
      i32.const 0
      local.get 3
      i32.store offset=1049004
      local.get 0
    )
    (func (;11;) (type 6) (param i32 i32 i32) (result i32)
      (local i32 i32 i32 i32 i32 i32 i32 i32)
      block ;; label = @1
        block ;; label = @2
          local.get 2
          i32.const 15
          i32.gt_u
          br_if 0 (;@2;)
          local.get 0
          local.set 3
          br 1 (;@1;)
        end
        local.get 0
        i32.const 0
        local.get 0
        i32.sub
        i32.const 3
        i32.and
        local.tee 4
        i32.add
        local.set 5
        block ;; label = @2
          local.get 4
          i32.eqz
          br_if 0 (;@2;)
          local.get 0
          local.set 3
          local.get 1
          local.set 6
          loop ;; label = @3
            local.get 3
            local.get 6
            i32.load8_u
            i32.store8
            local.get 6
            i32.const 1
            i32.add
            local.set 6
            local.get 3
            i32.const 1
            i32.add
            local.tee 3
            local.get 5
            i32.lt_u
            br_if 0 (;@3;)
          end
        end
        local.get 5
        local.get 2
        local.get 4
        i32.sub
        local.tee 7
        i32.const -4
        i32.and
        local.tee 8
        i32.add
        local.set 3
        block ;; label = @2
          block ;; label = @3
            local.get 1
            local.get 4
            i32.add
            local.tee 9
            i32.const 3
            i32.and
            i32.eqz
            br_if 0 (;@3;)
            local.get 8
            i32.const 1
            i32.lt_s
            br_if 1 (;@2;)
            local.get 9
            i32.const 3
            i32.shl
            local.tee 6
            i32.const 24
            i32.and
            local.set 2
            local.get 9
            i32.const -4
            i32.and
            local.tee 10
            i32.const 4
            i32.add
            local.set 1
            i32.const 0
            local.get 6
            i32.sub
            i32.const 24
            i32.and
            local.set 4
            local.get 10
            i32.load
            local.set 6
            loop ;; label = @4
              local.get 5
              local.get 6
              local.get 2
              i32.shr_u
              local.get 1
              i32.load
              local.tee 6
              local.get 4
              i32.shl
              i32.or
              i32.store
              local.get 1
              i32.const 4
              i32.add
              local.set 1
              local.get 5
              i32.const 4
              i32.add
              local.tee 5
              local.get 3
              i32.lt_u
              br_if 0 (;@4;)
              br 2 (;@2;)
            end
          end
          local.get 8
          i32.const 1
          i32.lt_s
          br_if 0 (;@2;)
          local.get 9
          local.set 1
          loop ;; label = @3
            local.get 5
            local.get 1
            i32.load
            i32.store
            local.get 1
            i32.const 4
            i32.add
            local.set 1
            local.get 5
            i32.const 4
            i32.add
            local.tee 5
            local.get 3
            i32.lt_u
            br_if 0 (;@3;)
          end
        end
        local.get 7
        i32.const 3
        i32.and
        local.set 2
        local.get 9
        local.get 8
        i32.add
        local.set 1
      end
      block ;; label = @1
        local.get 2
        i32.eqz
        br_if 0 (;@1;)
        local.get 3
        local.get 2
        i32.add
        local.set 5
        loop ;; label = @2
          local.get 3
          local.get 1
          i32.load8_u
          i32.store8
          local.get 1
          i32.const 1
          i32.add
          local.set 1
          local.get 3
          i32.const 1
          i32.add
          local.tee 3
          local.get 5
          i32.lt_u
          br_if 0 (;@2;)
        end
      end
      local.get 0
    )
    (func (;12;) (type 6) (param i32 i32 i32) (result i32)
      local.get 0
      local.get 1
      local.get 2
      call 11
    )
    (memory (;0;) 17)
    (global (;0;) (mut i32) i32.const 1048576)
    (global (;1;) i32 i32.const 1049030)
    (global (;2;) i32 i32.const 1049040)
    (export "memory" (memory 0))
    (export "add-two" (func 1))
    (export "cabi_realloc" (func 10))
    (export "__data_end" (global 1))
    (export "__heap_base" (global 2))
    (@producers
      (processed-by "wit-component" "0.14.2")
      (processed-by "wit-bindgen-rust" "0.12.0")
    )
  )
  (core instance (;0;) (instantiate 0))
  (alias core export 0 "memory" (core memory (;0;)))
  (alias core export 0 "cabi_realloc" (core func (;0;)))
  (type (;0;) (func (param "input" s32) (result s32)))
  (alias core export 0 "add-two" (core func (;1;)))
  (func (;0;) (type 0) (canon lift (core func 1)))
  (export (;1;) "add-two" (func 0))
  (@producers
    (processed-by "wit-component" "0.14.0")
  )
)

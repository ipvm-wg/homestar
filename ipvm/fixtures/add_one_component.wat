(component
  (core module (;0;)
    (type (;0;) (func (param i32) (result i32)))
    (type (;1;) (func))
    (type (;2;) (func (param i32 i32) (result i32)))
    (type (;3;) (func (param i32 i32 i32 i32) (result i32)))
    (type (;4;) (func (param i32 i32)))
    (type (;5;) (func (param i32)))
    (type (;6;) (func (result i32)))
    (type (;7;) (func (param i32 i32 i32)))
    (type (;8;) (func (param i32 i32 i32 i32 i32) (result i32)))
    (type (;9;) (func (param i32 i32 i32) (result i32)))
    (func (;0;) (type 0) (param i32) (result i32)
      local.get 0
      i32.const 1
      i32.add
    )
    (func (;1;) (type 1))
    (func (;2;) (type 2) (param i32 i32) (result i32)
      (local i32)
      local.get 0
      local.get 1
      call 12
      local.set 2
      local.get 2
      return
    )
    (func (;3;) (type 3) (param i32 i32 i32 i32) (result i32)
      (local i32)
      local.get 0
      local.get 1
      local.get 2
      local.get 3
      call 13
      local.set 4
      local.get 4
      return
    )
    (func (;4;) (type 2) (param i32 i32) (result i32)
      (local i32 i32 i32 i32 i32 i32)
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                local.get 1
                i32.const 9
                i32.lt_u
                br_if 0 (;@5;)
                i32.const 16
                i32.const 8
                call 14
                local.get 1
                i32.gt_u
                br_if 1 (;@4;)
                br 2 (;@3;)
              end
              local.get 0
              call 5
              local.set 2
              br 2 (;@2;)
            end
            i32.const 16
            i32.const 8
            call 14
            local.set 1
          end
          call 33
          local.tee 3
          i32.const 8
          call 14
          local.set 4
          i32.const 20
          i32.const 8
          call 14
          local.set 5
          i32.const 16
          i32.const 8
          call 14
          local.set 6
          i32.const 0
          local.set 2
          i32.const 0
          i32.const 16
          i32.const 8
          call 14
          i32.const 2
          i32.shl
          i32.sub
          local.tee 7
          local.get 3
          local.get 6
          local.get 4
          local.get 5
          i32.add
          i32.add
          i32.sub
          i32.const -65544
          i32.add
          i32.const -9
          i32.and
          i32.const -3
          i32.add
          local.tee 3
          local.get 7
          local.get 3
          i32.lt_u
          select
          local.get 1
          i32.sub
          local.get 0
          i32.le_u
          br_if 0 (;@2;)
          local.get 1
          i32.const 16
          local.get 0
          i32.const 4
          i32.add
          i32.const 16
          i32.const 8
          call 14
          i32.const -5
          i32.add
          local.get 0
          i32.gt_u
          select
          i32.const 8
          call 14
          local.tee 4
          i32.add
          i32.const 16
          i32.const 8
          call 14
          i32.add
          i32.const -4
          i32.add
          call 5
          local.tee 3
          i32.eqz
          br_if 0 (;@2;)
          local.get 3
          call 34
          local.set 0
          block ;; label = @3
            block ;; label = @4
              local.get 1
              i32.const -1
              i32.add
              local.tee 2
              local.get 3
              i32.and
              br_if 0 (;@4;)
              local.get 0
              local.set 1
              br 1 (;@3;)
            end
            local.get 2
            local.get 3
            i32.add
            i32.const 0
            local.get 1
            i32.sub
            i32.and
            call 34
            local.set 2
            i32.const 16
            i32.const 8
            call 14
            local.set 3
            local.get 0
            call 19
            local.get 2
            i32.const 0
            local.get 1
            local.get 2
            local.get 0
            i32.sub
            local.get 3
            i32.gt_u
            select
            i32.add
            local.tee 1
            local.get 0
            i32.sub
            local.tee 2
            i32.sub
            local.set 3
            block ;; label = @4
              local.get 0
              call 24
              br_if 0 (;@4;)
              local.get 1
              local.get 3
              call 25
              local.get 0
              local.get 2
              call 25
              local.get 0
              local.get 2
              call 6
              br 1 (;@3;)
            end
            local.get 0
            i32.load
            local.set 0
            local.get 1
            local.get 3
            i32.store offset=4
            local.get 1
            local.get 0
            local.get 2
            i32.add
            i32.store
          end
          local.get 1
          call 24
          br_if 1 (;@1;)
          local.get 1
          call 19
          local.tee 0
          i32.const 16
          i32.const 8
          call 14
          local.get 4
          i32.add
          i32.le_u
          br_if 1 (;@1;)
          local.get 1
          local.get 4
          call 30
          local.set 2
          local.get 1
          local.get 4
          call 25
          local.get 2
          local.get 0
          local.get 4
          i32.sub
          local.tee 0
          call 25
          local.get 2
          local.get 0
          call 6
          br 1 (;@1;)
        end
        local.get 2
        return
      end
      local.get 1
      call 32
      local.set 0
      local.get 1
      call 24
      drop
      local.get 0
    )
    (func (;5;) (type 0) (param i32) (result i32)
      (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i64)
      global.get 0
      i32.const 16
      i32.sub
      local.tee 1
      global.set 0
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 0
            i32.const 245
            i32.lt_u
            br_if 0 (;@3;)
            call 33
            local.tee 2
            i32.const 8
            call 14
            local.set 3
            i32.const 20
            i32.const 8
            call 14
            local.set 4
            i32.const 16
            i32.const 8
            call 14
            local.set 5
            i32.const 0
            local.set 6
            i32.const 0
            i32.const 16
            i32.const 8
            call 14
            i32.const 2
            i32.shl
            i32.sub
            local.tee 7
            local.get 2
            local.get 5
            local.get 3
            local.get 4
            i32.add
            i32.add
            i32.sub
            i32.const -65544
            i32.add
            i32.const -9
            i32.and
            i32.const -3
            i32.add
            local.tee 2
            local.get 7
            local.get 2
            i32.lt_u
            select
            local.get 0
            i32.le_u
            br_if 2 (;@1;)
            local.get 0
            i32.const 4
            i32.add
            i32.const 8
            call 14
            local.set 2
            i32.const 0
            i32.load offset=1048584
            i32.eqz
            br_if 1 (;@2;)
            i32.const 0
            local.set 8
            block ;; label = @4
              local.get 2
              i32.const 256
              i32.lt_u
              br_if 0 (;@4;)
              i32.const 31
              local.set 8
              local.get 2
              i32.const 16777215
              i32.gt_u
              br_if 0 (;@4;)
              local.get 2
              i32.const 6
              local.get 2
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
              local.set 8
            end
            i32.const 0
            local.get 2
            i32.sub
            local.set 6
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  local.get 8
                  i32.const 2
                  i32.shl
                  i32.const 1048852
                  i32.add
                  i32.load
                  local.tee 0
                  i32.eqz
                  br_if 0 (;@6;)
                  local.get 2
                  local.get 8
                  call 17
                  i32.shl
                  local.set 5
                  i32.const 0
                  local.set 4
                  i32.const 0
                  local.set 3
                  loop ;; label = @7
                    block ;; label = @8
                      local.get 0
                      call 36
                      call 19
                      local.tee 7
                      local.get 2
                      i32.lt_u
                      br_if 0 (;@8;)
                      local.get 7
                      local.get 2
                      i32.sub
                      local.tee 7
                      local.get 6
                      i32.ge_u
                      br_if 0 (;@8;)
                      local.get 7
                      local.set 6
                      local.get 0
                      local.set 3
                      local.get 7
                      br_if 0 (;@8;)
                      i32.const 0
                      local.set 6
                      local.get 0
                      local.set 3
                      br 3 (;@5;)
                    end
                    local.get 0
                    i32.const 20
                    i32.add
                    i32.load
                    local.tee 7
                    local.get 4
                    local.get 7
                    local.get 0
                    local.get 5
                    i32.const 29
                    i32.shr_u
                    i32.const 4
                    i32.and
                    i32.add
                    i32.const 16
                    i32.add
                    i32.load
                    local.tee 0
                    i32.ne
                    select
                    local.get 4
                    local.get 7
                    select
                    local.set 4
                    local.get 5
                    i32.const 1
                    i32.shl
                    local.set 5
                    local.get 0
                    br_if 0 (;@7;)
                  end
                  block ;; label = @7
                    local.get 4
                    i32.eqz
                    br_if 0 (;@7;)
                    local.get 4
                    local.set 0
                    br 2 (;@5;)
                  end
                  local.get 3
                  br_if 2 (;@4;)
                end
                i32.const 0
                local.set 3
                i32.const 1
                local.get 8
                i32.shl
                call 15
                i32.const 0
                i32.load offset=1048584
                i32.and
                local.tee 0
                i32.eqz
                br_if 3 (;@2;)
                local.get 0
                call 16
                i32.ctz
                i32.const 2
                i32.shl
                i32.const 1048852
                i32.add
                i32.load
                local.tee 0
                i32.eqz
                br_if 3 (;@2;)
              end
              loop ;; label = @5
                local.get 0
                local.get 3
                local.get 0
                call 36
                call 19
                local.tee 4
                local.get 2
                i32.ge_u
                local.get 4
                local.get 2
                i32.sub
                local.tee 4
                local.get 6
                i32.lt_u
                i32.and
                local.tee 5
                select
                local.set 3
                local.get 4
                local.get 6
                local.get 5
                select
                local.set 6
                local.get 0
                call 35
                local.tee 0
                br_if 0 (;@5;)
              end
              local.get 3
              i32.eqz
              br_if 2 (;@2;)
            end
            block ;; label = @4
              i32.const 0
              i32.load offset=1048980
              local.tee 0
              local.get 2
              i32.lt_u
              br_if 0 (;@4;)
              local.get 6
              local.get 0
              local.get 2
              i32.sub
              i32.ge_u
              br_if 2 (;@2;)
            end
            local.get 3
            call 36
            local.tee 0
            local.get 2
            call 30
            local.set 4
            local.get 3
            call 7
            block ;; label = @4
              block ;; label = @5
                local.get 6
                i32.const 16
                i32.const 8
                call 14
                i32.lt_u
                br_if 0 (;@5;)
                local.get 0
                local.get 2
                call 27
                local.get 4
                local.get 6
                call 28
                block ;; label = @6
                  local.get 6
                  i32.const 256
                  i32.lt_u
                  br_if 0 (;@6;)
                  local.get 4
                  local.get 6
                  call 8
                  br 2 (;@4;)
                end
                local.get 6
                i32.const -8
                i32.and
                i32.const 1048588
                i32.add
                local.set 3
                block ;; label = @6
                  block ;; label = @7
                    i32.const 0
                    i32.load offset=1048580
                    local.tee 5
                    i32.const 1
                    local.get 6
                    i32.const 3
                    i32.shr_u
                    i32.shl
                    local.tee 6
                    i32.and
                    i32.eqz
                    br_if 0 (;@7;)
                    local.get 3
                    i32.load offset=8
                    local.set 6
                    br 1 (;@6;)
                  end
                  i32.const 0
                  local.get 5
                  local.get 6
                  i32.or
                  i32.store offset=1048580
                  local.get 3
                  local.set 6
                end
                local.get 3
                local.get 4
                i32.store offset=8
                local.get 6
                local.get 4
                i32.store offset=12
                local.get 4
                local.get 3
                i32.store offset=12
                local.get 4
                local.get 6
                i32.store offset=8
                br 1 (;@4;)
              end
              local.get 0
              local.get 6
              local.get 2
              i32.add
              call 26
            end
            local.get 0
            call 32
            local.tee 6
            i32.eqz
            br_if 1 (;@2;)
            br 2 (;@1;)
          end
          i32.const 16
          local.get 0
          i32.const 4
          i32.add
          i32.const 16
          i32.const 8
          call 14
          i32.const -5
          i32.add
          local.get 0
          i32.gt_u
          select
          i32.const 8
          call 14
          local.set 2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  block ;; label = @7
                    block ;; label = @8
                      block ;; label = @9
                        i32.const 0
                        i32.load offset=1048580
                        local.tee 4
                        local.get 2
                        i32.const 3
                        i32.shr_u
                        local.tee 6
                        i32.shr_u
                        local.tee 0
                        i32.const 3
                        i32.and
                        br_if 0 (;@9;)
                        local.get 2
                        i32.const 0
                        i32.load offset=1048980
                        i32.le_u
                        br_if 7 (;@2;)
                        local.get 0
                        br_if 1 (;@8;)
                        i32.const 0
                        i32.load offset=1048584
                        local.tee 0
                        i32.eqz
                        br_if 7 (;@2;)
                        local.get 0
                        call 16
                        i32.ctz
                        i32.const 2
                        i32.shl
                        i32.const 1048852
                        i32.add
                        i32.load
                        local.tee 3
                        call 36
                        call 19
                        local.get 2
                        i32.sub
                        local.set 6
                        block ;; label = @10
                          local.get 3
                          call 35
                          local.tee 0
                          i32.eqz
                          br_if 0 (;@10;)
                          loop ;; label = @11
                            local.get 0
                            call 36
                            call 19
                            local.get 2
                            i32.sub
                            local.tee 4
                            local.get 6
                            local.get 4
                            local.get 6
                            i32.lt_u
                            local.tee 4
                            select
                            local.set 6
                            local.get 0
                            local.get 3
                            local.get 4
                            select
                            local.set 3
                            local.get 0
                            call 35
                            local.tee 0
                            br_if 0 (;@11;)
                          end
                        end
                        local.get 3
                        call 36
                        local.tee 0
                        local.get 2
                        call 30
                        local.set 4
                        local.get 3
                        call 7
                        local.get 6
                        i32.const 16
                        i32.const 8
                        call 14
                        i32.lt_u
                        br_if 5 (;@4;)
                        local.get 4
                        call 36
                        local.set 4
                        local.get 0
                        local.get 2
                        call 27
                        local.get 4
                        local.get 6
                        call 28
                        i32.const 0
                        i32.load offset=1048980
                        local.tee 7
                        i32.eqz
                        br_if 4 (;@5;)
                        local.get 7
                        i32.const -8
                        i32.and
                        i32.const 1048588
                        i32.add
                        local.set 5
                        i32.const 0
                        i32.load offset=1048988
                        local.set 3
                        i32.const 0
                        i32.load offset=1048580
                        local.tee 8
                        i32.const 1
                        local.get 7
                        i32.const 3
                        i32.shr_u
                        i32.shl
                        local.tee 7
                        i32.and
                        i32.eqz
                        br_if 2 (;@7;)
                        local.get 5
                        i32.load offset=8
                        local.set 7
                        br 3 (;@6;)
                      end
                      block ;; label = @9
                        block ;; label = @10
                          local.get 0
                          i32.const -1
                          i32.xor
                          i32.const 1
                          i32.and
                          local.get 6
                          i32.add
                          local.tee 2
                          i32.const 3
                          i32.shl
                          local.tee 3
                          i32.const 1048596
                          i32.add
                          i32.load
                          local.tee 0
                          i32.const 8
                          i32.add
                          i32.load
                          local.tee 6
                          local.get 3
                          i32.const 1048588
                          i32.add
                          local.tee 3
                          i32.eq
                          br_if 0 (;@10;)
                          local.get 6
                          local.get 3
                          i32.store offset=12
                          local.get 3
                          local.get 6
                          i32.store offset=8
                          br 1 (;@9;)
                        end
                        i32.const 0
                        local.get 4
                        i32.const -2
                        local.get 2
                        i32.rotl
                        i32.and
                        i32.store offset=1048580
                      end
                      local.get 0
                      local.get 2
                      i32.const 3
                      i32.shl
                      call 26
                      local.get 0
                      call 32
                      local.set 6
                      br 7 (;@1;)
                    end
                    block ;; label = @8
                      block ;; label = @9
                        i32.const 1
                        local.get 6
                        i32.const 31
                        i32.and
                        local.tee 6
                        i32.shl
                        call 15
                        local.get 0
                        local.get 6
                        i32.shl
                        i32.and
                        call 16
                        i32.ctz
                        local.tee 6
                        i32.const 3
                        i32.shl
                        local.tee 4
                        i32.const 1048596
                        i32.add
                        i32.load
                        local.tee 0
                        i32.const 8
                        i32.add
                        i32.load
                        local.tee 3
                        local.get 4
                        i32.const 1048588
                        i32.add
                        local.tee 4
                        i32.eq
                        br_if 0 (;@9;)
                        local.get 3
                        local.get 4
                        i32.store offset=12
                        local.get 4
                        local.get 3
                        i32.store offset=8
                        br 1 (;@8;)
                      end
                      i32.const 0
                      i32.const 0
                      i32.load offset=1048580
                      i32.const -2
                      local.get 6
                      i32.rotl
                      i32.and
                      i32.store offset=1048580
                    end
                    local.get 0
                    local.get 2
                    call 27
                    local.get 0
                    local.get 2
                    call 30
                    local.tee 4
                    local.get 6
                    i32.const 3
                    i32.shl
                    local.get 2
                    i32.sub
                    local.tee 5
                    call 28
                    block ;; label = @8
                      i32.const 0
                      i32.load offset=1048980
                      local.tee 3
                      i32.eqz
                      br_if 0 (;@8;)
                      local.get 3
                      i32.const -8
                      i32.and
                      i32.const 1048588
                      i32.add
                      local.set 6
                      i32.const 0
                      i32.load offset=1048988
                      local.set 2
                      block ;; label = @9
                        block ;; label = @10
                          i32.const 0
                          i32.load offset=1048580
                          local.tee 7
                          i32.const 1
                          local.get 3
                          i32.const 3
                          i32.shr_u
                          i32.shl
                          local.tee 3
                          i32.and
                          i32.eqz
                          br_if 0 (;@10;)
                          local.get 6
                          i32.load offset=8
                          local.set 3
                          br 1 (;@9;)
                        end
                        i32.const 0
                        local.get 7
                        local.get 3
                        i32.or
                        i32.store offset=1048580
                        local.get 6
                        local.set 3
                      end
                      local.get 6
                      local.get 2
                      i32.store offset=8
                      local.get 3
                      local.get 2
                      i32.store offset=12
                      local.get 2
                      local.get 6
                      i32.store offset=12
                      local.get 2
                      local.get 3
                      i32.store offset=8
                    end
                    i32.const 0
                    local.get 4
                    i32.store offset=1048988
                    i32.const 0
                    local.get 5
                    i32.store offset=1048980
                    local.get 0
                    call 32
                    local.set 6
                    br 6 (;@1;)
                  end
                  i32.const 0
                  local.get 8
                  local.get 7
                  i32.or
                  i32.store offset=1048580
                  local.get 5
                  local.set 7
                end
                local.get 5
                local.get 3
                i32.store offset=8
                local.get 7
                local.get 3
                i32.store offset=12
                local.get 3
                local.get 5
                i32.store offset=12
                local.get 3
                local.get 7
                i32.store offset=8
              end
              i32.const 0
              local.get 4
              i32.store offset=1048988
              i32.const 0
              local.get 6
              i32.store offset=1048980
              br 1 (;@3;)
            end
            local.get 0
            local.get 6
            local.get 2
            i32.add
            call 26
          end
          local.get 0
          call 32
          local.tee 6
          br_if 1 (;@1;)
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
                          i32.const 0
                          i32.load offset=1048980
                          local.tee 6
                          local.get 2
                          i32.ge_u
                          br_if 0 (;@10;)
                          i32.const 0
                          i32.load offset=1048984
                          local.tee 0
                          local.get 2
                          i32.gt_u
                          br_if 2 (;@8;)
                          local.get 1
                          i32.const 1048580
                          local.get 2
                          call 33
                          local.tee 0
                          i32.sub
                          local.get 0
                          i32.const 8
                          call 14
                          i32.add
                          i32.const 20
                          i32.const 8
                          call 14
                          i32.add
                          i32.const 16
                          i32.const 8
                          call 14
                          i32.add
                          i32.const 8
                          i32.add
                          i32.const 65536
                          call 14
                          call 43
                          local.get 1
                          i32.load
                          local.tee 6
                          br_if 1 (;@9;)
                          i32.const 0
                          local.set 6
                          br 9 (;@1;)
                        end
                        i32.const 0
                        i32.load offset=1048988
                        local.set 0
                        block ;; label = @10
                          local.get 6
                          local.get 2
                          i32.sub
                          local.tee 6
                          i32.const 16
                          i32.const 8
                          call 14
                          i32.ge_u
                          br_if 0 (;@10;)
                          i32.const 0
                          i32.const 0
                          i32.store offset=1048988
                          i32.const 0
                          i32.load offset=1048980
                          local.set 2
                          i32.const 0
                          i32.const 0
                          i32.store offset=1048980
                          local.get 0
                          local.get 2
                          call 26
                          local.get 0
                          call 32
                          local.set 6
                          br 9 (;@1;)
                        end
                        local.get 0
                        local.get 2
                        call 30
                        local.set 3
                        i32.const 0
                        local.get 6
                        i32.store offset=1048980
                        i32.const 0
                        local.get 3
                        i32.store offset=1048988
                        local.get 3
                        local.get 6
                        call 28
                        local.get 0
                        local.get 2
                        call 27
                        local.get 0
                        call 32
                        local.set 6
                        br 8 (;@1;)
                      end
                      local.get 1
                      i32.load offset=8
                      local.set 8
                      i32.const 0
                      i32.const 0
                      i32.load offset=1048996
                      local.get 1
                      i32.load offset=4
                      local.tee 5
                      i32.add
                      local.tee 0
                      i32.store offset=1048996
                      i32.const 0
                      i32.const 0
                      i32.load offset=1049000
                      local.tee 3
                      local.get 0
                      local.get 3
                      local.get 0
                      i32.gt_u
                      select
                      i32.store offset=1049000
                      block ;; label = @9
                        block ;; label = @10
                          block ;; label = @11
                            i32.const 0
                            i32.load offset=1048992
                            i32.eqz
                            br_if 0 (;@11;)
                            i32.const 1049004
                            local.set 0
                            loop ;; label = @12
                              local.get 6
                              local.get 0
                              call 42
                              i32.eq
                              br_if 2 (;@10;)
                              local.get 0
                              i32.load offset=8
                              local.tee 0
                              br_if 0 (;@12;)
                              br 3 (;@9;)
                            end
                          end
                          i32.const 0
                          i32.load offset=1049024
                          local.tee 0
                          i32.eqz
                          br_if 3 (;@7;)
                          local.get 6
                          local.get 0
                          i32.lt_u
                          br_if 3 (;@7;)
                          br 7 (;@3;)
                        end
                        local.get 0
                        call 39
                        br_if 0 (;@9;)
                        local.get 0
                        call 40
                        local.get 8
                        i32.ne
                        br_if 0 (;@9;)
                        local.get 0
                        i32.const 0
                        i32.load offset=1048992
                        call 41
                        br_if 3 (;@6;)
                      end
                      i32.const 0
                      i32.const 0
                      i32.load offset=1049024
                      local.tee 0
                      local.get 6
                      local.get 6
                      local.get 0
                      i32.gt_u
                      select
                      i32.store offset=1049024
                      local.get 6
                      local.get 5
                      i32.add
                      local.set 3
                      i32.const 1049004
                      local.set 0
                      block ;; label = @9
                        block ;; label = @10
                          block ;; label = @11
                            loop ;; label = @12
                              local.get 0
                              i32.load
                              local.get 3
                              i32.eq
                              br_if 1 (;@11;)
                              local.get 0
                              i32.load offset=8
                              local.tee 0
                              br_if 0 (;@12;)
                              br 2 (;@10;)
                            end
                          end
                          local.get 0
                          call 39
                          br_if 0 (;@10;)
                          local.get 0
                          call 40
                          local.get 8
                          i32.eq
                          br_if 1 (;@9;)
                        end
                        i32.const 0
                        i32.load offset=1048992
                        local.set 3
                        i32.const 1049004
                        local.set 0
                        block ;; label = @10
                          loop ;; label = @11
                            block ;; label = @12
                              local.get 0
                              i32.load
                              local.get 3
                              i32.gt_u
                              br_if 0 (;@12;)
                              local.get 0
                              call 42
                              local.get 3
                              i32.gt_u
                              br_if 2 (;@10;)
                            end
                            local.get 0
                            i32.load offset=8
                            local.tee 0
                            br_if 0 (;@11;)
                          end
                          i32.const 0
                          local.set 0
                        end
                        local.get 0
                        call 42
                        local.tee 4
                        i32.const 20
                        i32.const 8
                        call 14
                        local.tee 9
                        i32.sub
                        i32.const -23
                        i32.add
                        local.set 0
                        local.get 3
                        local.get 0
                        local.get 0
                        call 32
                        local.tee 7
                        i32.const 8
                        call 14
                        local.get 7
                        i32.sub
                        i32.add
                        local.tee 0
                        local.get 0
                        local.get 3
                        i32.const 16
                        i32.const 8
                        call 14
                        i32.add
                        i32.lt_u
                        select
                        local.tee 7
                        call 32
                        local.set 10
                        local.get 7
                        local.get 9
                        call 30
                        local.set 0
                        call 33
                        local.tee 11
                        i32.const 8
                        call 14
                        local.set 12
                        i32.const 20
                        i32.const 8
                        call 14
                        local.set 13
                        i32.const 16
                        i32.const 8
                        call 14
                        local.set 14
                        i32.const 0
                        local.get 6
                        local.get 6
                        call 32
                        local.tee 15
                        i32.const 8
                        call 14
                        local.get 15
                        i32.sub
                        local.tee 16
                        call 30
                        local.tee 15
                        i32.store offset=1048992
                        i32.const 0
                        local.get 11
                        local.get 5
                        i32.add
                        local.get 14
                        local.get 12
                        local.get 13
                        i32.add
                        i32.add
                        local.get 16
                        i32.add
                        i32.sub
                        local.tee 11
                        i32.store offset=1048984
                        local.get 15
                        local.get 11
                        i32.const 1
                        i32.or
                        i32.store offset=4
                        call 33
                        local.tee 12
                        i32.const 8
                        call 14
                        local.set 13
                        i32.const 20
                        i32.const 8
                        call 14
                        local.set 14
                        i32.const 16
                        i32.const 8
                        call 14
                        local.set 16
                        local.get 15
                        local.get 11
                        call 30
                        local.get 16
                        local.get 14
                        local.get 13
                        local.get 12
                        i32.sub
                        i32.add
                        i32.add
                        i32.store offset=4
                        i32.const 0
                        i32.const 2097152
                        i32.store offset=1049020
                        local.get 7
                        local.get 9
                        call 27
                        i32.const 0
                        i64.load offset=1049004 align=4
                        local.set 17
                        local.get 10
                        i32.const 8
                        i32.add
                        i32.const 0
                        i64.load offset=1049012 align=4
                        i64.store align=4
                        local.get 10
                        local.get 17
                        i64.store align=4
                        i32.const 0
                        local.get 8
                        i32.store offset=1049016
                        i32.const 0
                        local.get 5
                        i32.store offset=1049008
                        i32.const 0
                        local.get 6
                        i32.store offset=1049004
                        i32.const 0
                        local.get 10
                        i32.store offset=1049012
                        loop ;; label = @10
                          local.get 0
                          i32.const 4
                          call 30
                          local.set 6
                          local.get 0
                          call 18
                          i32.store offset=4
                          local.get 6
                          local.set 0
                          local.get 6
                          i32.const 4
                          i32.add
                          local.get 4
                          i32.lt_u
                          br_if 0 (;@10;)
                        end
                        local.get 7
                        local.get 3
                        i32.eq
                        br_if 7 (;@2;)
                        local.get 7
                        local.get 3
                        i32.sub
                        local.set 0
                        local.get 3
                        local.get 0
                        local.get 3
                        local.get 0
                        call 30
                        call 29
                        block ;; label = @10
                          local.get 0
                          i32.const 256
                          i32.lt_u
                          br_if 0 (;@10;)
                          local.get 3
                          local.get 0
                          call 8
                          br 8 (;@2;)
                        end
                        local.get 0
                        i32.const -8
                        i32.and
                        i32.const 1048588
                        i32.add
                        local.set 6
                        block ;; label = @10
                          block ;; label = @11
                            i32.const 0
                            i32.load offset=1048580
                            local.tee 4
                            i32.const 1
                            local.get 0
                            i32.const 3
                            i32.shr_u
                            i32.shl
                            local.tee 0
                            i32.and
                            i32.eqz
                            br_if 0 (;@11;)
                            local.get 6
                            i32.load offset=8
                            local.set 0
                            br 1 (;@10;)
                          end
                          i32.const 0
                          local.get 4
                          local.get 0
                          i32.or
                          i32.store offset=1048580
                          local.get 6
                          local.set 0
                        end
                        local.get 6
                        local.get 3
                        i32.store offset=8
                        local.get 0
                        local.get 3
                        i32.store offset=12
                        local.get 3
                        local.get 6
                        i32.store offset=12
                        local.get 3
                        local.get 0
                        i32.store offset=8
                        br 7 (;@2;)
                      end
                      local.get 0
                      i32.load
                      local.set 4
                      local.get 0
                      local.get 6
                      i32.store
                      local.get 0
                      local.get 0
                      i32.load offset=4
                      local.get 5
                      i32.add
                      i32.store offset=4
                      local.get 6
                      call 32
                      local.tee 0
                      i32.const 8
                      call 14
                      local.set 3
                      local.get 4
                      call 32
                      local.tee 5
                      i32.const 8
                      call 14
                      local.set 7
                      local.get 6
                      local.get 3
                      local.get 0
                      i32.sub
                      i32.add
                      local.tee 6
                      local.get 2
                      call 30
                      local.set 3
                      local.get 6
                      local.get 2
                      call 27
                      local.get 4
                      local.get 7
                      local.get 5
                      i32.sub
                      i32.add
                      local.tee 0
                      local.get 2
                      local.get 6
                      i32.add
                      i32.sub
                      local.set 2
                      block ;; label = @9
                        local.get 0
                        i32.const 0
                        i32.load offset=1048992
                        i32.eq
                        br_if 0 (;@9;)
                        local.get 0
                        i32.const 0
                        i32.load offset=1048988
                        i32.eq
                        br_if 4 (;@5;)
                        local.get 0
                        call 23
                        br_if 5 (;@4;)
                        block ;; label = @10
                          block ;; label = @11
                            local.get 0
                            call 19
                            local.tee 4
                            i32.const 256
                            i32.lt_u
                            br_if 0 (;@11;)
                            local.get 0
                            call 7
                            br 1 (;@10;)
                          end
                          block ;; label = @11
                            local.get 0
                            i32.const 12
                            i32.add
                            i32.load
                            local.tee 5
                            local.get 0
                            i32.const 8
                            i32.add
                            i32.load
                            local.tee 7
                            i32.eq
                            br_if 0 (;@11;)
                            local.get 7
                            local.get 5
                            i32.store offset=12
                            local.get 5
                            local.get 7
                            i32.store offset=8
                            br 1 (;@10;)
                          end
                          i32.const 0
                          i32.const 0
                          i32.load offset=1048580
                          i32.const -2
                          local.get 4
                          i32.const 3
                          i32.shr_u
                          i32.rotl
                          i32.and
                          i32.store offset=1048580
                        end
                        local.get 4
                        local.get 2
                        i32.add
                        local.set 2
                        local.get 0
                        local.get 4
                        call 30
                        local.set 0
                        br 5 (;@4;)
                      end
                      i32.const 0
                      local.get 3
                      i32.store offset=1048992
                      i32.const 0
                      i32.const 0
                      i32.load offset=1048984
                      local.get 2
                      i32.add
                      local.tee 0
                      i32.store offset=1048984
                      local.get 3
                      local.get 0
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      local.get 6
                      call 32
                      local.set 6
                      br 7 (;@1;)
                    end
                    i32.const 0
                    local.get 0
                    local.get 2
                    i32.sub
                    local.tee 6
                    i32.store offset=1048984
                    i32.const 0
                    i32.const 0
                    i32.load offset=1048992
                    local.tee 0
                    local.get 2
                    call 30
                    local.tee 3
                    i32.store offset=1048992
                    local.get 3
                    local.get 6
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    local.get 0
                    local.get 2
                    call 27
                    local.get 0
                    call 32
                    local.set 6
                    br 6 (;@1;)
                  end
                  i32.const 0
                  local.get 6
                  i32.store offset=1049024
                  br 3 (;@3;)
                end
                local.get 0
                local.get 0
                i32.load offset=4
                local.get 5
                i32.add
                i32.store offset=4
                i32.const 0
                i32.load offset=1048992
                i32.const 0
                i32.load offset=1048984
                local.get 5
                i32.add
                call 11
                br 3 (;@2;)
              end
              i32.const 0
              local.get 3
              i32.store offset=1048988
              i32.const 0
              i32.const 0
              i32.load offset=1048980
              local.get 2
              i32.add
              local.tee 0
              i32.store offset=1048980
              local.get 3
              local.get 0
              call 28
              local.get 6
              call 32
              local.set 6
              br 3 (;@1;)
            end
            local.get 3
            local.get 2
            local.get 0
            call 29
            block ;; label = @4
              local.get 2
              i32.const 256
              i32.lt_u
              br_if 0 (;@4;)
              local.get 3
              local.get 2
              call 8
              local.get 6
              call 32
              local.set 6
              br 3 (;@1;)
            end
            local.get 2
            i32.const -8
            i32.and
            i32.const 1048588
            i32.add
            local.set 0
            block ;; label = @4
              block ;; label = @5
                i32.const 0
                i32.load offset=1048580
                local.tee 4
                i32.const 1
                local.get 2
                i32.const 3
                i32.shr_u
                i32.shl
                local.tee 2
                i32.and
                i32.eqz
                br_if 0 (;@5;)
                local.get 0
                i32.load offset=8
                local.set 2
                br 1 (;@4;)
              end
              i32.const 0
              local.get 4
              local.get 2
              i32.or
              i32.store offset=1048580
              local.get 0
              local.set 2
            end
            local.get 0
            local.get 3
            i32.store offset=8
            local.get 2
            local.get 3
            i32.store offset=12
            local.get 3
            local.get 0
            i32.store offset=12
            local.get 3
            local.get 2
            i32.store offset=8
            local.get 6
            call 32
            local.set 6
            br 2 (;@1;)
          end
          i32.const 0
          i32.const 4095
          i32.store offset=1049028
          i32.const 0
          local.get 8
          i32.store offset=1049016
          i32.const 0
          local.get 5
          i32.store offset=1049008
          i32.const 0
          local.get 6
          i32.store offset=1049004
          i32.const 0
          i32.const 1048588
          i32.store offset=1048600
          i32.const 0
          i32.const 1048596
          i32.store offset=1048608
          i32.const 0
          i32.const 1048588
          i32.store offset=1048596
          i32.const 0
          i32.const 1048604
          i32.store offset=1048616
          i32.const 0
          i32.const 1048596
          i32.store offset=1048604
          i32.const 0
          i32.const 1048612
          i32.store offset=1048624
          i32.const 0
          i32.const 1048604
          i32.store offset=1048612
          i32.const 0
          i32.const 1048620
          i32.store offset=1048632
          i32.const 0
          i32.const 1048612
          i32.store offset=1048620
          i32.const 0
          i32.const 1048628
          i32.store offset=1048640
          i32.const 0
          i32.const 1048620
          i32.store offset=1048628
          i32.const 0
          i32.const 1048636
          i32.store offset=1048648
          i32.const 0
          i32.const 1048628
          i32.store offset=1048636
          i32.const 0
          i32.const 1048644
          i32.store offset=1048656
          i32.const 0
          i32.const 1048636
          i32.store offset=1048644
          i32.const 0
          i32.const 1048652
          i32.store offset=1048664
          i32.const 0
          i32.const 1048644
          i32.store offset=1048652
          i32.const 0
          i32.const 1048652
          i32.store offset=1048660
          i32.const 0
          i32.const 1048660
          i32.store offset=1048672
          i32.const 0
          i32.const 1048660
          i32.store offset=1048668
          i32.const 0
          i32.const 1048668
          i32.store offset=1048680
          i32.const 0
          i32.const 1048668
          i32.store offset=1048676
          i32.const 0
          i32.const 1048676
          i32.store offset=1048688
          i32.const 0
          i32.const 1048676
          i32.store offset=1048684
          i32.const 0
          i32.const 1048684
          i32.store offset=1048696
          i32.const 0
          i32.const 1048684
          i32.store offset=1048692
          i32.const 0
          i32.const 1048692
          i32.store offset=1048704
          i32.const 0
          i32.const 1048692
          i32.store offset=1048700
          i32.const 0
          i32.const 1048700
          i32.store offset=1048712
          i32.const 0
          i32.const 1048700
          i32.store offset=1048708
          i32.const 0
          i32.const 1048708
          i32.store offset=1048720
          i32.const 0
          i32.const 1048708
          i32.store offset=1048716
          i32.const 0
          i32.const 1048716
          i32.store offset=1048728
          i32.const 0
          i32.const 1048724
          i32.store offset=1048736
          i32.const 0
          i32.const 1048716
          i32.store offset=1048724
          i32.const 0
          i32.const 1048732
          i32.store offset=1048744
          i32.const 0
          i32.const 1048724
          i32.store offset=1048732
          i32.const 0
          i32.const 1048740
          i32.store offset=1048752
          i32.const 0
          i32.const 1048732
          i32.store offset=1048740
          i32.const 0
          i32.const 1048748
          i32.store offset=1048760
          i32.const 0
          i32.const 1048740
          i32.store offset=1048748
          i32.const 0
          i32.const 1048756
          i32.store offset=1048768
          i32.const 0
          i32.const 1048748
          i32.store offset=1048756
          i32.const 0
          i32.const 1048764
          i32.store offset=1048776
          i32.const 0
          i32.const 1048756
          i32.store offset=1048764
          i32.const 0
          i32.const 1048772
          i32.store offset=1048784
          i32.const 0
          i32.const 1048764
          i32.store offset=1048772
          i32.const 0
          i32.const 1048780
          i32.store offset=1048792
          i32.const 0
          i32.const 1048772
          i32.store offset=1048780
          i32.const 0
          i32.const 1048788
          i32.store offset=1048800
          i32.const 0
          i32.const 1048780
          i32.store offset=1048788
          i32.const 0
          i32.const 1048796
          i32.store offset=1048808
          i32.const 0
          i32.const 1048788
          i32.store offset=1048796
          i32.const 0
          i32.const 1048804
          i32.store offset=1048816
          i32.const 0
          i32.const 1048796
          i32.store offset=1048804
          i32.const 0
          i32.const 1048812
          i32.store offset=1048824
          i32.const 0
          i32.const 1048804
          i32.store offset=1048812
          i32.const 0
          i32.const 1048820
          i32.store offset=1048832
          i32.const 0
          i32.const 1048812
          i32.store offset=1048820
          i32.const 0
          i32.const 1048828
          i32.store offset=1048840
          i32.const 0
          i32.const 1048820
          i32.store offset=1048828
          i32.const 0
          i32.const 1048836
          i32.store offset=1048848
          i32.const 0
          i32.const 1048828
          i32.store offset=1048836
          i32.const 0
          i32.const 1048836
          i32.store offset=1048844
          call 33
          local.tee 3
          i32.const 8
          call 14
          local.set 4
          i32.const 20
          i32.const 8
          call 14
          local.set 7
          i32.const 16
          i32.const 8
          call 14
          local.set 8
          i32.const 0
          local.get 6
          local.get 6
          call 32
          local.tee 0
          i32.const 8
          call 14
          local.get 0
          i32.sub
          local.tee 10
          call 30
          local.tee 0
          i32.store offset=1048992
          i32.const 0
          local.get 3
          local.get 5
          i32.add
          local.get 8
          local.get 4
          local.get 7
          i32.add
          i32.add
          local.get 10
          i32.add
          i32.sub
          local.tee 6
          i32.store offset=1048984
          local.get 0
          local.get 6
          i32.const 1
          i32.or
          i32.store offset=4
          call 33
          local.tee 3
          i32.const 8
          call 14
          local.set 4
          i32.const 20
          i32.const 8
          call 14
          local.set 5
          i32.const 16
          i32.const 8
          call 14
          local.set 7
          local.get 0
          local.get 6
          call 30
          local.get 7
          local.get 5
          local.get 4
          local.get 3
          i32.sub
          i32.add
          i32.add
          i32.store offset=4
          i32.const 0
          i32.const 2097152
          i32.store offset=1049020
        end
        i32.const 0
        local.set 6
        i32.const 0
        i32.load offset=1048984
        local.tee 0
        local.get 2
        i32.le_u
        br_if 0 (;@1;)
        i32.const 0
        local.get 0
        local.get 2
        i32.sub
        local.tee 6
        i32.store offset=1048984
        i32.const 0
        i32.const 0
        i32.load offset=1048992
        local.tee 0
        local.get 2
        call 30
        local.tee 3
        i32.store offset=1048992
        local.get 3
        local.get 6
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 0
        local.get 2
        call 27
        local.get 0
        call 32
        local.set 6
      end
      local.get 1
      i32.const 16
      i32.add
      global.set 0
      local.get 6
    )
    (func (;6;) (type 4) (param i32 i32)
      (local i32 i32 i32 i32)
      local.get 0
      local.get 1
      call 30
      local.set 2
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 0
            call 21
            br_if 0 (;@3;)
            local.get 0
            i32.load
            local.set 3
            block ;; label = @4
              block ;; label = @5
                local.get 0
                call 24
                br_if 0 (;@5;)
                local.get 3
                local.get 1
                i32.add
                local.set 1
                local.get 0
                local.get 3
                call 31
                local.tee 0
                i32.const 0
                i32.load offset=1048988
                i32.ne
                br_if 1 (;@4;)
                local.get 2
                i32.load offset=4
                i32.const 3
                i32.and
                i32.const 3
                i32.ne
                br_if 2 (;@3;)
                i32.const 0
                local.get 1
                i32.store offset=1048980
                local.get 0
                local.get 1
                local.get 2
                call 29
                return
              end
              i32.const 1048580
              local.get 0
              local.get 3
              i32.sub
              local.get 3
              local.get 1
              i32.add
              i32.const 16
              i32.add
              local.tee 0
              call 46
              i32.eqz
              br_if 2 (;@2;)
              i32.const 0
              i32.const 0
              i32.load offset=1048996
              local.get 0
              i32.sub
              i32.store offset=1048996
              return
            end
            block ;; label = @4
              local.get 3
              i32.const 256
              i32.lt_u
              br_if 0 (;@4;)
              local.get 0
              call 7
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
            i32.load offset=1048580
            i32.const -2
            local.get 3
            i32.const 3
            i32.shr_u
            i32.rotl
            i32.and
            i32.store offset=1048580
          end
          block ;; label = @3
            local.get 2
            call 20
            i32.eqz
            br_if 0 (;@3;)
            local.get 0
            local.get 1
            local.get 2
            call 29
            br 2 (;@1;)
          end
          block ;; label = @3
            block ;; label = @4
              local.get 2
              i32.const 0
              i32.load offset=1048992
              i32.eq
              br_if 0 (;@4;)
              local.get 2
              i32.const 0
              i32.load offset=1048988
              i32.ne
              br_if 1 (;@3;)
              i32.const 0
              local.get 0
              i32.store offset=1048988
              i32.const 0
              i32.const 0
              i32.load offset=1048980
              local.get 1
              i32.add
              local.tee 1
              i32.store offset=1048980
              local.get 0
              local.get 1
              call 28
              return
            end
            i32.const 0
            local.get 0
            i32.store offset=1048992
            i32.const 0
            i32.const 0
            i32.load offset=1048984
            local.get 1
            i32.add
            local.tee 1
            i32.store offset=1048984
            local.get 0
            local.get 1
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 0
            i32.const 0
            i32.load offset=1048988
            i32.ne
            br_if 1 (;@2;)
            i32.const 0
            i32.const 0
            i32.store offset=1048980
            i32.const 0
            i32.const 0
            i32.store offset=1048988
            return
          end
          local.get 2
          call 19
          local.tee 3
          local.get 1
          i32.add
          local.set 1
          block ;; label = @3
            block ;; label = @4
              local.get 3
              i32.const 256
              i32.lt_u
              br_if 0 (;@4;)
              local.get 2
              call 7
              br 1 (;@3;)
            end
            block ;; label = @4
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
              br_if 0 (;@4;)
              local.get 2
              local.get 4
              i32.store offset=12
              local.get 4
              local.get 2
              i32.store offset=8
              br 1 (;@3;)
            end
            i32.const 0
            i32.const 0
            i32.load offset=1048580
            i32.const -2
            local.get 3
            i32.const 3
            i32.shr_u
            i32.rotl
            i32.and
            i32.store offset=1048580
          end
          local.get 0
          local.get 1
          call 28
          local.get 0
          i32.const 0
          i32.load offset=1048988
          i32.ne
          br_if 1 (;@1;)
          i32.const 0
          local.get 1
          i32.store offset=1048980
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
        call 8
        return
      end
      local.get 1
      i32.const -8
      i32.and
      i32.const 1048588
      i32.add
      local.set 2
      block ;; label = @1
        block ;; label = @2
          i32.const 0
          i32.load offset=1048580
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
        i32.store offset=1048580
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
    (func (;7;) (type 5) (param i32)
      (local i32 i32 i32 i32 i32)
      local.get 0
      i32.load offset=24
      local.set 1
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 0
            call 37
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
            local.set 3
            br 2 (;@1;)
          end
          local.get 0
          call 38
          local.tee 4
          local.get 0
          call 37
          local.tee 3
          call 36
          i32.store offset=12
          local.get 3
          local.get 4
          call 36
          i32.store offset=8
          br 1 (;@1;)
        end
        local.get 2
        local.get 0
        i32.const 16
        i32.add
        local.get 3
        select
        local.set 2
        loop ;; label = @2
          local.get 2
          local.set 5
          block ;; label = @3
            local.get 4
            local.tee 3
            i32.const 20
            i32.add
            local.tee 2
            i32.load
            local.tee 4
            br_if 0 (;@3;)
            local.get 3
            i32.const 16
            i32.add
            local.set 2
            local.get 3
            i32.load offset=16
            local.set 4
          end
          local.get 4
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
            i32.const 1048852
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
            local.get 3
            i32.store
            local.get 3
            br_if 1 (;@2;)
            br 2 (;@1;)
          end
          local.get 4
          local.get 3
          i32.store
          local.get 3
          br_if 0 (;@2;)
          i32.const 0
          i32.const 0
          i32.load offset=1048584
          i32.const -2
          local.get 0
          i32.load offset=28
          i32.rotl
          i32.and
          i32.store offset=1048584
          return
        end
        local.get 3
        local.get 1
        i32.store offset=24
        block ;; label = @2
          local.get 0
          i32.load offset=16
          local.tee 4
          i32.eqz
          br_if 0 (;@2;)
          local.get 3
          local.get 4
          i32.store offset=16
          local.get 4
          local.get 3
          i32.store offset=24
        end
        local.get 0
        i32.const 20
        i32.add
        i32.load
        local.tee 4
        i32.eqz
        br_if 0 (;@1;)
        local.get 3
        i32.const 20
        i32.add
        local.get 4
        i32.store
        local.get 4
        local.get 3
        i32.store offset=24
        return
      end
    )
    (func (;8;) (type 4) (param i32 i32)
      (local i32 i32 i32 i32 i32)
      i32.const 0
      local.set 2
      block ;; label = @1
        local.get 1
        i32.const 256
        i32.lt_u
        br_if 0 (;@1;)
        i32.const 31
        local.set 2
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
      i32.const 1048852
      i32.add
      local.set 3
      local.get 0
      call 36
      local.set 4
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                i32.const 0
                i32.load offset=1048584
                local.tee 5
                i32.const 1
                local.get 2
                i32.shl
                local.tee 6
                i32.and
                i32.eqz
                br_if 0 (;@5;)
                local.get 3
                i32.load
                local.set 5
                local.get 2
                call 17
                local.set 2
                local.get 5
                call 36
                call 19
                local.get 1
                i32.ne
                br_if 1 (;@4;)
                local.get 5
                local.set 2
                br 2 (;@3;)
              end
              i32.const 0
              local.get 5
              local.get 6
              i32.or
              i32.store offset=1048584
              local.get 3
              local.get 0
              i32.store
              local.get 0
              local.get 3
              i32.store offset=24
              br 3 (;@1;)
            end
            local.get 1
            local.get 2
            i32.shl
            local.set 3
            loop ;; label = @4
              local.get 5
              local.get 3
              i32.const 29
              i32.shr_u
              i32.const 4
              i32.and
              i32.add
              i32.const 16
              i32.add
              local.tee 6
              i32.load
              local.tee 2
              i32.eqz
              br_if 2 (;@2;)
              local.get 3
              i32.const 1
              i32.shl
              local.set 3
              local.get 2
              local.set 5
              local.get 2
              call 36
              call 19
              local.get 1
              i32.ne
              br_if 0 (;@4;)
            end
          end
          local.get 2
          call 36
          local.tee 2
          i32.load offset=8
          local.tee 3
          local.get 4
          i32.store offset=12
          local.get 2
          local.get 4
          i32.store offset=8
          local.get 4
          local.get 2
          i32.store offset=12
          local.get 4
          local.get 3
          i32.store offset=8
          local.get 0
          i32.const 0
          i32.store offset=24
          return
        end
        local.get 6
        local.get 0
        i32.store
        local.get 0
        local.get 5
        i32.store offset=24
      end
      local.get 4
      local.get 4
      i32.store offset=8
      local.get 4
      local.get 4
      i32.store offset=12
    )
    (func (;9;) (type 6) (result i32)
      (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
      i32.const 0
      local.set 0
      i32.const 0
      local.set 1
      block ;; label = @1
        i32.const 0
        i32.load offset=1049012
        local.tee 2
        i32.eqz
        br_if 0 (;@1;)
        i32.const 1049004
        local.set 3
        i32.const 0
        local.set 1
        i32.const 0
        local.set 0
        loop ;; label = @2
          local.get 2
          local.tee 4
          i32.load offset=8
          local.set 2
          local.get 4
          i32.load offset=4
          local.set 5
          local.get 4
          i32.load
          local.set 6
          block ;; label = @3
            block ;; label = @4
              i32.const 1048580
              local.get 4
              i32.const 12
              i32.add
              i32.load
              i32.const 1
              i32.shr_u
              call 47
              i32.eqz
              br_if 0 (;@4;)
              local.get 4
              call 39
              br_if 0 (;@4;)
              local.get 6
              local.get 6
              call 32
              local.tee 7
              i32.const 8
              call 14
              local.get 7
              i32.sub
              i32.add
              local.tee 7
              call 19
              local.set 8
              call 33
              local.tee 9
              i32.const 8
              call 14
              local.set 10
              i32.const 20
              i32.const 8
              call 14
              local.set 11
              i32.const 16
              i32.const 8
              call 14
              local.set 12
              local.get 7
              call 23
              br_if 0 (;@4;)
              local.get 7
              local.get 8
              i32.add
              local.get 6
              local.get 9
              local.get 5
              i32.add
              local.get 10
              local.get 11
              i32.add
              local.get 12
              i32.add
              i32.sub
              i32.add
              i32.lt_u
              br_if 0 (;@4;)
              block ;; label = @5
                block ;; label = @6
                  local.get 7
                  i32.const 0
                  i32.load offset=1048988
                  i32.eq
                  br_if 0 (;@6;)
                  local.get 7
                  call 7
                  br 1 (;@5;)
                end
                i32.const 0
                i32.const 0
                i32.store offset=1048980
                i32.const 0
                i32.const 0
                i32.store offset=1048988
              end
              block ;; label = @5
                i32.const 1048580
                local.get 6
                local.get 5
                call 46
                br_if 0 (;@5;)
                local.get 7
                local.get 8
                call 8
                br 1 (;@4;)
              end
              i32.const 0
              i32.const 0
              i32.load offset=1048996
              local.get 5
              i32.sub
              i32.store offset=1048996
              local.get 3
              local.get 2
              i32.store offset=8
              local.get 5
              local.get 1
              i32.add
              local.set 1
              br 1 (;@3;)
            end
            local.get 4
            local.set 3
          end
          local.get 0
          i32.const 1
          i32.add
          local.set 0
          local.get 2
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
      i32.store offset=1049028
      local.get 1
    )
    (func (;10;) (type 5) (param i32)
      (local i32 i32 i32 i32 i32 i32)
      local.get 0
      call 34
      local.set 0
      local.get 0
      local.get 0
      call 19
      local.tee 1
      call 30
      local.set 2
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 0
            call 21
            br_if 0 (;@3;)
            local.get 0
            i32.load
            local.set 3
            block ;; label = @4
              block ;; label = @5
                local.get 0
                call 24
                br_if 0 (;@5;)
                local.get 3
                local.get 1
                i32.add
                local.set 1
                local.get 0
                local.get 3
                call 31
                local.tee 0
                i32.const 0
                i32.load offset=1048988
                i32.ne
                br_if 1 (;@4;)
                local.get 2
                i32.load offset=4
                i32.const 3
                i32.and
                i32.const 3
                i32.ne
                br_if 2 (;@3;)
                i32.const 0
                local.get 1
                i32.store offset=1048980
                local.get 0
                local.get 1
                local.get 2
                call 29
                return
              end
              i32.const 1048580
              local.get 0
              local.get 3
              i32.sub
              local.get 3
              local.get 1
              i32.add
              i32.const 16
              i32.add
              local.tee 0
              call 46
              i32.eqz
              br_if 2 (;@2;)
              i32.const 0
              i32.const 0
              i32.load offset=1048996
              local.get 0
              i32.sub
              i32.store offset=1048996
              return
            end
            block ;; label = @4
              local.get 3
              i32.const 256
              i32.lt_u
              br_if 0 (;@4;)
              local.get 0
              call 7
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
            i32.load offset=1048580
            i32.const -2
            local.get 3
            i32.const 3
            i32.shr_u
            i32.rotl
            i32.and
            i32.store offset=1048580
          end
          block ;; label = @3
            block ;; label = @4
              local.get 2
              call 20
              i32.eqz
              br_if 0 (;@4;)
              local.get 0
              local.get 1
              local.get 2
              call 29
              br 1 (;@3;)
            end
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  block ;; label = @7
                    local.get 2
                    i32.const 0
                    i32.load offset=1048992
                    i32.eq
                    br_if 0 (;@7;)
                    local.get 2
                    i32.const 0
                    i32.load offset=1048988
                    i32.ne
                    br_if 1 (;@6;)
                    i32.const 0
                    local.get 0
                    i32.store offset=1048988
                    i32.const 0
                    i32.const 0
                    i32.load offset=1048980
                    local.get 1
                    i32.add
                    local.tee 1
                    i32.store offset=1048980
                    local.get 0
                    local.get 1
                    call 28
                    return
                  end
                  i32.const 0
                  local.get 0
                  i32.store offset=1048992
                  i32.const 0
                  i32.const 0
                  i32.load offset=1048984
                  local.get 1
                  i32.add
                  local.tee 1
                  i32.store offset=1048984
                  local.get 0
                  local.get 1
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 0
                  i32.const 0
                  i32.load offset=1048988
                  i32.eq
                  br_if 1 (;@5;)
                  br 2 (;@4;)
                end
                local.get 2
                call 19
                local.tee 3
                local.get 1
                i32.add
                local.set 1
                block ;; label = @6
                  block ;; label = @7
                    local.get 3
                    i32.const 256
                    i32.lt_u
                    br_if 0 (;@7;)
                    local.get 2
                    call 7
                    br 1 (;@6;)
                  end
                  block ;; label = @7
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
                    br_if 0 (;@7;)
                    local.get 2
                    local.get 4
                    i32.store offset=12
                    local.get 4
                    local.get 2
                    i32.store offset=8
                    br 1 (;@6;)
                  end
                  i32.const 0
                  i32.const 0
                  i32.load offset=1048580
                  i32.const -2
                  local.get 3
                  i32.const 3
                  i32.shr_u
                  i32.rotl
                  i32.and
                  i32.store offset=1048580
                end
                local.get 0
                local.get 1
                call 28
                local.get 0
                i32.const 0
                i32.load offset=1048988
                i32.ne
                br_if 2 (;@3;)
                i32.const 0
                local.get 1
                i32.store offset=1048980
                br 3 (;@2;)
              end
              i32.const 0
              i32.const 0
              i32.store offset=1048980
              i32.const 0
              i32.const 0
              i32.store offset=1048988
            end
            i32.const 0
            i32.load offset=1049020
            local.get 1
            i32.ge_u
            br_if 1 (;@2;)
            call 33
            local.tee 0
            i32.const 8
            call 14
            local.set 1
            i32.const 20
            i32.const 8
            call 14
            local.set 2
            i32.const 16
            i32.const 8
            call 14
            local.set 3
            i32.const 0
            i32.const 16
            i32.const 8
            call 14
            i32.const 2
            i32.shl
            i32.sub
            local.tee 4
            local.get 0
            local.get 3
            local.get 1
            local.get 2
            i32.add
            i32.add
            i32.sub
            i32.const -65544
            i32.add
            i32.const -9
            i32.and
            i32.const -3
            i32.add
            local.tee 0
            local.get 4
            local.get 0
            i32.lt_u
            select
            i32.eqz
            br_if 1 (;@2;)
            i32.const 0
            i32.load offset=1048992
            i32.eqz
            br_if 1 (;@2;)
            call 33
            local.tee 0
            i32.const 8
            call 14
            local.set 1
            i32.const 20
            i32.const 8
            call 14
            local.set 3
            i32.const 16
            i32.const 8
            call 14
            local.set 4
            i32.const 0
            local.set 2
            block ;; label = @4
              i32.const 0
              i32.load offset=1048984
              local.tee 5
              local.get 4
              local.get 3
              local.get 1
              local.get 0
              i32.sub
              i32.add
              i32.add
              local.tee 0
              i32.le_u
              br_if 0 (;@4;)
              local.get 5
              local.get 0
              i32.const -1
              i32.xor
              i32.add
              i32.const -65536
              i32.and
              local.set 3
              i32.const 0
              i32.load offset=1048992
              local.set 1
              i32.const 1049004
              local.set 0
              block ;; label = @5
                loop ;; label = @6
                  block ;; label = @7
                    local.get 0
                    i32.load
                    local.get 1
                    i32.gt_u
                    br_if 0 (;@7;)
                    local.get 0
                    call 42
                    local.get 1
                    i32.gt_u
                    br_if 2 (;@5;)
                  end
                  local.get 0
                  i32.load offset=8
                  local.tee 0
                  br_if 0 (;@6;)
                end
                i32.const 0
                local.set 0
              end
              i32.const 0
              local.set 2
              local.get 0
              call 39
              br_if 0 (;@4;)
              i32.const 1048580
              local.get 0
              i32.const 12
              i32.add
              i32.load
              i32.const 1
              i32.shr_u
              call 47
              i32.eqz
              br_if 0 (;@4;)
              local.get 0
              i32.load offset=4
              local.get 3
              i32.lt_u
              br_if 0 (;@4;)
              i32.const 1049004
              local.set 1
              loop ;; label = @5
                local.get 0
                local.get 1
                call 41
                br_if 1 (;@4;)
                local.get 1
                i32.load offset=8
                local.tee 1
                br_if 0 (;@5;)
              end
              i32.const 1048580
              local.get 0
              i32.load
              local.get 0
              i32.load offset=4
              local.tee 1
              local.get 1
              local.get 3
              i32.sub
              call 45
              i32.eqz
              br_if 0 (;@4;)
              local.get 3
              i32.eqz
              br_if 0 (;@4;)
              local.get 0
              local.get 0
              i32.load offset=4
              local.get 3
              i32.sub
              i32.store offset=4
              i32.const 0
              i32.const 0
              i32.load offset=1048996
              local.get 3
              i32.sub
              i32.store offset=1048996
              i32.const 0
              i32.load offset=1048984
              local.set 1
              i32.const 0
              i32.load offset=1048992
              local.set 0
              i32.const 0
              local.get 0
              local.get 0
              call 32
              local.tee 2
              i32.const 8
              call 14
              local.get 2
              i32.sub
              local.tee 2
              call 30
              local.tee 0
              i32.store offset=1048992
              i32.const 0
              local.get 1
              local.get 3
              local.get 2
              i32.add
              i32.sub
              local.tee 1
              i32.store offset=1048984
              local.get 0
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              call 33
              local.tee 2
              i32.const 8
              call 14
              local.set 4
              i32.const 20
              i32.const 8
              call 14
              local.set 5
              i32.const 16
              i32.const 8
              call 14
              local.set 6
              local.get 0
              local.get 1
              call 30
              local.get 6
              local.get 5
              local.get 4
              local.get 2
              i32.sub
              i32.add
              i32.add
              i32.store offset=4
              i32.const 0
              i32.const 2097152
              i32.store offset=1049020
              local.get 3
              local.set 2
            end
            local.get 2
            i32.const 0
            call 9
            i32.sub
            i32.ne
            br_if 1 (;@2;)
            i32.const 0
            i32.load offset=1048984
            i32.const 0
            i32.load offset=1049020
            i32.le_u
            br_if 1 (;@2;)
            i32.const 0
            i32.const -1
            i32.store offset=1049020
            return
          end
          local.get 1
          i32.const 256
          i32.lt_u
          br_if 1 (;@1;)
          local.get 0
          local.get 1
          call 8
          i32.const 0
          i32.const 0
          i32.load offset=1049028
          i32.const -1
          i32.add
          local.tee 0
          i32.store offset=1049028
          local.get 0
          br_if 0 (;@2;)
          call 9
          drop
          return
        end
        return
      end
      local.get 1
      i32.const -8
      i32.and
      i32.const 1048588
      i32.add
      local.set 2
      block ;; label = @1
        block ;; label = @2
          i32.const 0
          i32.load offset=1048580
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
        i32.store offset=1048580
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
    (func (;11;) (type 4) (param i32 i32)
      (local i32 i32 i32 i32)
      local.get 0
      local.get 0
      call 32
      local.tee 2
      i32.const 8
      call 14
      local.get 2
      i32.sub
      local.tee 2
      call 30
      local.set 0
      i32.const 0
      local.get 1
      local.get 2
      i32.sub
      local.tee 1
      i32.store offset=1048984
      i32.const 0
      local.get 0
      i32.store offset=1048992
      local.get 0
      local.get 1
      i32.const 1
      i32.or
      i32.store offset=4
      call 33
      local.tee 2
      i32.const 8
      call 14
      local.set 3
      i32.const 20
      i32.const 8
      call 14
      local.set 4
      i32.const 16
      i32.const 8
      call 14
      local.set 5
      local.get 0
      local.get 1
      call 30
      local.get 5
      local.get 4
      local.get 3
      local.get 2
      i32.sub
      i32.add
      i32.add
      i32.store offset=4
      i32.const 0
      i32.const 2097152
      i32.store offset=1049020
    )
    (func (;12;) (type 2) (param i32 i32) (result i32)
      local.get 0
      local.get 1
      call 4
    )
    (func (;13;) (type 3) (param i32 i32 i32 i32) (result i32)
      (local i32 i32 i32 i32 i32 i32)
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              local.get 2
              i32.const 9
              i32.lt_u
              br_if 0 (;@4;)
              local.get 3
              local.get 2
              call 4
              local.tee 2
              br_if 1 (;@3;)
              i32.const 0
              return
            end
            call 33
            local.tee 1
            i32.const 8
            call 14
            local.set 4
            i32.const 20
            i32.const 8
            call 14
            local.set 5
            i32.const 16
            i32.const 8
            call 14
            local.set 6
            i32.const 0
            local.set 2
            i32.const 0
            i32.const 16
            i32.const 8
            call 14
            i32.const 2
            i32.shl
            i32.sub
            local.tee 7
            local.get 1
            local.get 6
            local.get 4
            local.get 5
            i32.add
            i32.add
            i32.sub
            i32.const -65544
            i32.add
            i32.const -9
            i32.and
            i32.const -3
            i32.add
            local.tee 1
            local.get 7
            local.get 1
            i32.lt_u
            select
            local.get 3
            i32.le_u
            br_if 1 (;@2;)
            i32.const 16
            local.get 3
            i32.const 4
            i32.add
            i32.const 16
            i32.const 8
            call 14
            i32.const -5
            i32.add
            local.get 3
            i32.gt_u
            select
            i32.const 8
            call 14
            local.set 4
            local.get 0
            call 34
            local.set 1
            local.get 1
            local.get 1
            call 19
            local.tee 5
            call 30
            local.set 6
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  block ;; label = @7
                    block ;; label = @8
                      block ;; label = @9
                        block ;; label = @10
                          block ;; label = @11
                            local.get 1
                            call 24
                            br_if 0 (;@11;)
                            local.get 5
                            local.get 4
                            i32.ge_u
                            br_if 1 (;@10;)
                            local.get 6
                            i32.const 0
                            i32.load offset=1048992
                            i32.eq
                            br_if 2 (;@9;)
                            local.get 6
                            i32.const 0
                            i32.load offset=1048988
                            i32.eq
                            br_if 3 (;@8;)
                            local.get 6
                            call 20
                            br_if 7 (;@4;)
                            local.get 6
                            call 19
                            local.tee 7
                            local.get 5
                            i32.add
                            local.tee 5
                            local.get 4
                            i32.lt_u
                            br_if 7 (;@4;)
                            local.get 5
                            local.get 4
                            i32.sub
                            local.set 8
                            local.get 7
                            i32.const 256
                            i32.lt_u
                            br_if 4 (;@7;)
                            local.get 6
                            call 7
                            br 5 (;@6;)
                          end
                          local.get 1
                          call 19
                          local.set 5
                          local.get 4
                          i32.const 256
                          i32.lt_u
                          br_if 6 (;@4;)
                          block ;; label = @11
                            local.get 5
                            local.get 4
                            i32.const 4
                            i32.add
                            i32.lt_u
                            br_if 0 (;@11;)
                            local.get 5
                            local.get 4
                            i32.sub
                            i32.const 131073
                            i32.lt_u
                            br_if 6 (;@5;)
                          end
                          i32.const 1048580
                          local.get 1
                          local.get 1
                          i32.load
                          local.tee 6
                          i32.sub
                          local.get 5
                          local.get 6
                          i32.add
                          i32.const 16
                          i32.add
                          local.tee 7
                          local.get 4
                          i32.const 31
                          i32.add
                          i32.const 1048580
                          call 48
                          call 14
                          local.tee 5
                          i32.const 1
                          call 44
                          local.tee 4
                          i32.eqz
                          br_if 6 (;@4;)
                          local.get 4
                          local.get 6
                          i32.add
                          local.tee 1
                          local.get 5
                          local.get 6
                          i32.sub
                          local.tee 3
                          i32.const -16
                          i32.add
                          local.tee 2
                          i32.store offset=4
                          call 18
                          local.set 0
                          local.get 1
                          local.get 2
                          call 30
                          local.get 0
                          i32.store offset=4
                          local.get 1
                          local.get 3
                          i32.const -12
                          i32.add
                          call 30
                          i32.const 0
                          i32.store offset=4
                          i32.const 0
                          i32.const 0
                          i32.load offset=1048996
                          local.get 5
                          local.get 7
                          i32.sub
                          i32.add
                          local.tee 3
                          i32.store offset=1048996
                          i32.const 0
                          i32.const 0
                          i32.load offset=1049024
                          local.tee 2
                          local.get 4
                          local.get 4
                          local.get 2
                          i32.gt_u
                          select
                          i32.store offset=1049024
                          i32.const 0
                          i32.const 0
                          i32.load offset=1049000
                          local.tee 2
                          local.get 3
                          local.get 2
                          local.get 3
                          i32.gt_u
                          select
                          i32.store offset=1049000
                          br 9 (;@1;)
                        end
                        local.get 5
                        local.get 4
                        i32.sub
                        local.tee 5
                        i32.const 16
                        i32.const 8
                        call 14
                        i32.lt_u
                        br_if 4 (;@5;)
                        local.get 1
                        local.get 4
                        call 30
                        local.set 6
                        local.get 1
                        local.get 4
                        call 25
                        local.get 6
                        local.get 5
                        call 25
                        local.get 6
                        local.get 5
                        call 6
                        br 4 (;@5;)
                      end
                      i32.const 0
                      i32.load offset=1048984
                      local.get 5
                      i32.add
                      local.tee 5
                      local.get 4
                      i32.le_u
                      br_if 4 (;@4;)
                      local.get 1
                      local.get 4
                      call 30
                      local.set 6
                      local.get 1
                      local.get 4
                      call 25
                      local.get 6
                      local.get 5
                      local.get 4
                      i32.sub
                      local.tee 4
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      i32.const 0
                      local.get 4
                      i32.store offset=1048984
                      i32.const 0
                      local.get 6
                      i32.store offset=1048992
                      br 3 (;@5;)
                    end
                    i32.const 0
                    i32.load offset=1048980
                    local.get 5
                    i32.add
                    local.tee 5
                    local.get 4
                    i32.lt_u
                    br_if 3 (;@4;)
                    block ;; label = @8
                      block ;; label = @9
                        local.get 5
                        local.get 4
                        i32.sub
                        local.tee 6
                        i32.const 16
                        i32.const 8
                        call 14
                        i32.ge_u
                        br_if 0 (;@9;)
                        local.get 1
                        local.get 5
                        call 25
                        i32.const 0
                        local.set 6
                        i32.const 0
                        local.set 5
                        br 1 (;@8;)
                      end
                      local.get 1
                      local.get 4
                      call 30
                      local.tee 5
                      local.get 6
                      call 30
                      local.set 7
                      local.get 1
                      local.get 4
                      call 25
                      local.get 5
                      local.get 6
                      call 28
                      local.get 7
                      call 22
                    end
                    i32.const 0
                    local.get 5
                    i32.store offset=1048988
                    i32.const 0
                    local.get 6
                    i32.store offset=1048980
                    br 2 (;@5;)
                  end
                  block ;; label = @7
                    local.get 6
                    i32.const 12
                    i32.add
                    i32.load
                    local.tee 9
                    local.get 6
                    i32.const 8
                    i32.add
                    i32.load
                    local.tee 6
                    i32.eq
                    br_if 0 (;@7;)
                    local.get 6
                    local.get 9
                    i32.store offset=12
                    local.get 9
                    local.get 6
                    i32.store offset=8
                    br 1 (;@6;)
                  end
                  i32.const 0
                  i32.const 0
                  i32.load offset=1048580
                  i32.const -2
                  local.get 7
                  i32.const 3
                  i32.shr_u
                  i32.rotl
                  i32.and
                  i32.store offset=1048580
                end
                block ;; label = @6
                  local.get 8
                  i32.const 16
                  i32.const 8
                  call 14
                  i32.lt_u
                  br_if 0 (;@6;)
                  local.get 1
                  local.get 4
                  call 30
                  local.set 5
                  local.get 1
                  local.get 4
                  call 25
                  local.get 5
                  local.get 8
                  call 25
                  local.get 5
                  local.get 8
                  call 6
                  br 1 (;@5;)
                end
                local.get 1
                local.get 5
                call 25
              end
              local.get 1
              br_if 3 (;@1;)
            end
            local.get 3
            call 5
            local.tee 4
            i32.eqz
            br_if 1 (;@2;)
            local.get 4
            local.get 0
            local.get 1
            call 19
            i32.const -8
            i32.const -4
            local.get 1
            call 24
            select
            i32.add
            local.tee 2
            local.get 3
            local.get 2
            local.get 3
            i32.lt_u
            select
            call 49
            local.set 3
            local.get 0
            call 10
            local.get 3
            return
          end
          local.get 2
          local.get 0
          local.get 1
          local.get 3
          local.get 1
          local.get 3
          i32.lt_u
          select
          call 49
          drop
          local.get 0
          call 10
        end
        local.get 2
        return
      end
      local.get 1
      call 24
      drop
      local.get 1
      call 32
    )
    (func (;14;) (type 2) (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.add
      i32.const -1
      i32.add
      i32.const 0
      local.get 1
      i32.sub
      i32.and
    )
    (func (;15;) (type 0) (param i32) (result i32)
      local.get 0
      i32.const 1
      i32.shl
      local.tee 0
      i32.const 0
      local.get 0
      i32.sub
      i32.or
    )
    (func (;16;) (type 0) (param i32) (result i32)
      i32.const 0
      local.get 0
      i32.sub
      local.get 0
      i32.and
    )
    (func (;17;) (type 0) (param i32) (result i32)
      i32.const 0
      i32.const 25
      local.get 0
      i32.const 1
      i32.shr_u
      i32.sub
      local.get 0
      i32.const 31
      i32.eq
      select
    )
    (func (;18;) (type 6) (result i32)
      i32.const 7
    )
    (func (;19;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load offset=4
      i32.const -8
      i32.and
    )
    (func (;20;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load8_u offset=4
      i32.const 2
      i32.and
      i32.const 1
      i32.shr_u
    )
    (func (;21;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load offset=4
      i32.const 1
      i32.and
    )
    (func (;22;) (type 5) (param i32)
      local.get 0
      local.get 0
      i32.load offset=4
      i32.const -2
      i32.and
      i32.store offset=4
    )
    (func (;23;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load offset=4
      i32.const 3
      i32.and
      i32.const 1
      i32.ne
    )
    (func (;24;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load8_u offset=4
      i32.const 3
      i32.and
      i32.eqz
    )
    (func (;25;) (type 4) (param i32 i32)
      local.get 0
      local.get 0
      i32.load offset=4
      i32.const 1
      i32.and
      local.get 1
      i32.or
      i32.const 2
      i32.or
      i32.store offset=4
      local.get 0
      local.get 1
      i32.add
      local.tee 0
      local.get 0
      i32.load offset=4
      i32.const 1
      i32.or
      i32.store offset=4
    )
    (func (;26;) (type 4) (param i32 i32)
      local.get 0
      local.get 1
      i32.const 3
      i32.or
      i32.store offset=4
      local.get 0
      local.get 1
      i32.add
      local.tee 1
      local.get 1
      i32.load offset=4
      i32.const 1
      i32.or
      i32.store offset=4
    )
    (func (;27;) (type 4) (param i32 i32)
      local.get 0
      local.get 1
      i32.const 3
      i32.or
      i32.store offset=4
    )
    (func (;28;) (type 4) (param i32 i32)
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
    )
    (func (;29;) (type 7) (param i32 i32 i32)
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
      local.get 0
      local.get 1
      i32.add
      local.get 1
      i32.store
    )
    (func (;30;) (type 2) (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.add
    )
    (func (;31;) (type 2) (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.sub
    )
    (func (;32;) (type 0) (param i32) (result i32)
      local.get 0
      i32.const 8
      i32.add
    )
    (func (;33;) (type 6) (result i32)
      i32.const 8
    )
    (func (;34;) (type 0) (param i32) (result i32)
      local.get 0
      i32.const -8
      i32.add
    )
    (func (;35;) (type 0) (param i32) (result i32)
      (local i32)
      block ;; label = @1
        local.get 0
        i32.load offset=16
        local.tee 1
        br_if 0 (;@1;)
        local.get 0
        i32.const 20
        i32.add
        i32.load
        local.set 1
      end
      local.get 1
    )
    (func (;36;) (type 0) (param i32) (result i32)
      local.get 0
    )
    (func (;37;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load offset=12
    )
    (func (;38;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load offset=8
    )
    (func (;39;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load offset=12
      i32.const 1
      i32.and
    )
    (func (;40;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load offset=12
      i32.const 1
      i32.shr_u
    )
    (func (;41;) (type 2) (param i32 i32) (result i32)
      (local i32 i32)
      i32.const 0
      local.set 2
      block ;; label = @1
        local.get 0
        i32.load
        local.tee 3
        local.get 1
        i32.gt_u
        br_if 0 (;@1;)
        local.get 3
        local.get 0
        i32.load offset=4
        i32.add
        local.get 1
        i32.gt_u
        local.set 2
      end
      local.get 2
    )
    (func (;42;) (type 0) (param i32) (result i32)
      local.get 0
      i32.load
      local.get 0
      i32.load offset=4
      i32.add
    )
    (func (;43;) (type 7) (param i32 i32 i32)
      (local i32)
      local.get 2
      i32.const 16
      i32.shr_u
      memory.grow
      local.set 3
      local.get 0
      i32.const 0
      i32.store offset=8
      local.get 0
      i32.const 0
      local.get 2
      i32.const -65536
      i32.and
      local.get 3
      i32.const -1
      i32.eq
      local.tee 2
      select
      i32.store offset=4
      local.get 0
      i32.const 0
      local.get 3
      i32.const 16
      i32.shl
      local.get 2
      select
      i32.store
    )
    (func (;44;) (type 8) (param i32 i32 i32 i32 i32) (result i32)
      i32.const 0
    )
    (func (;45;) (type 3) (param i32 i32 i32 i32) (result i32)
      i32.const 0
    )
    (func (;46;) (type 9) (param i32 i32 i32) (result i32)
      i32.const 0
    )
    (func (;47;) (type 2) (param i32 i32) (result i32)
      i32.const 0
    )
    (func (;48;) (type 0) (param i32) (result i32)
      i32.const 65536
    )
    (func (;49;) (type 9) (param i32 i32 i32) (result i32)
      local.get 0
      local.get 1
      local.get 2
      call 50
    )
    (func (;50;) (type 9) (param i32 i32 i32) (result i32)
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
            local.tee 6
            i32.eqz
            br_if 0 (;@3;)
            local.get 8
            i32.const 1
            i32.lt_s
            br_if 1 (;@2;)
            local.get 9
            i32.const -4
            i32.and
            local.tee 10
            i32.const 4
            i32.add
            local.set 1
            i32.const 0
            local.get 6
            i32.const 3
            i32.shl
            local.tee 2
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
    (func (;51;) (type 3) (param i32 i32 i32 i32) (result i32)
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 1
            br_if 0 (;@3;)
            local.get 3
            i32.eqz
            br_if 2 (;@1;)
            local.get 3
            local.get 2
            call 2
            local.set 2
            br 1 (;@2;)
          end
          local.get 0
          local.get 1
          local.get 2
          local.get 3
          call 3
          local.set 2
        end
        local.get 2
        br_if 0 (;@1;)
        unreachable
        unreachable
      end
      local.get 2
    )
    (table (;0;) 2 2 funcref)
    (memory (;0;) 17)
    (global (;0;) (mut i32) i32.const 1048576)
    (global (;1;) i32 i32.const 1049032)
    (global (;2;) i32 i32.const 1049040)
    (export "memory" (memory 0))
    (export "add-one" (func 0))
    (export "cabi_realloc" (func 51))
    (export "__data_end" (global 1))
    (export "__heap_base" (global 2))
    (elem (;0;) (i32.const 1) func 1)
    (data (;0;) (i32.const 1048576) "\01\00\00\00")
  )
  (core instance (;0;) (instantiate 0))
  (alias core export 0 "memory" (core memory (;0;)))
  (alias core export 0 "cabi_realloc" (core func (;0;)))
  (type (;0;) (func (param "a" s32) (result s32)))
  (alias core export 0 "add-one" (core func (;1;)))
  (func (;0;) (type 0) (canon lift (core func 1)))
  (export (;1;) "add-one" (func 0))
)
